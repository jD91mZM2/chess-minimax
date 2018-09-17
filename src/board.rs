use arrayvec::ArrayVec;
use crate::{
    piece::{Piece, PieceKind},
    Pos,
    Side,
};
use std::mem;

pub type Change = ArrayVec<[Undo; 6]>;

/// Describes how to revert a change to the board. Used for undoing moves.
#[derive(Clone, Debug)]
pub enum Undo {
    Set(Pos, Option<Piece>),
    EnPassant(Option<Pos>),
    Castling(Side, Castling)
}

/// Information about whether or not one side can do a castling
#[derive(Clone, Copy, Debug)]
pub struct Castling {
    pub queenside: bool,
    pub kingside: bool
}
impl Default for Castling {
    fn default() -> Self {
        Self {
            queenside: true,
            kingside: true
        }
    }
}

/// The width (and height, because square) of the board
pub const WIDTH: i8 = 8;

fn edge_offset(side: Side, y: i8) -> i8 {
    match side {
        Side::Black => y,
        Side::White => (WIDTH - 1) - y
    }
}
fn is_castling(piece: Option<Piece>, m: Pos) -> bool {
    let Pos(rel_x, _) = m;
    piece.map(|p| p.kind == PieceKind::King).unwrap_or(false)
        && rel_x.abs() == 2
}

/// A typical chess board
#[derive(Debug, Clone)]
pub struct Board {
    pub(crate) pieces: [[Option<Piece>; WIDTH as usize]; WIDTH as usize],
    pub(crate) en_passant: Option<Pos>,
    pub(crate) castling_black: Castling,
    pub(crate) castling_white: Castling
}
impl Default for Board {
    fn default() -> Self {
        use crate::piece::PieceKind::{self, *};

        #[inline(always)]
        fn black(kind: PieceKind) -> Option<Piece> {
            Some(Piece::black(kind))
        }
        #[inline(always)]
        fn white(kind: PieceKind) -> Option<Piece> {
            Some(Piece::white(kind))
        }

        Self {
            pieces: [
//[white(King), None,          None,          None,         None,        None,          None,          None],
//[None,        None,          None,          None,         None,        None,          None,          None],
//[None,        None,          None,          None,         None,        None,          None,          None],
//[None,        None,          None,          None,         black(Rook), None,          black(Rook),   None],
//[None,        None,          None,          None,         None,        None,          None,          None],
//[None,        None,          None,          None,         None,        None,          None,          None],
//[None,        None,          None,          None,         None,        None,          None,          None],
//[None,        None,          None,          None,         None,        None,          None,          black(King)],
[black(Rook), black(Knight), black(Bishop), black(Queen), black(King), black(Bishop), black(Knight), black(Rook)],
[black(Pawn), black(Pawn),   black(Pawn),   black(Pawn),  black(Pawn), black(Pawn),   black(Pawn),   black(Pawn)],
[None,        None,          None,          None,         None,        None,          None,          None],
[None,        None,          None,          None,         None,        None,          None,          None],
[None,        None,          None,          None,         None,        None,          None,          None],
[None,        None,          None,          None,         None,        None,          None,          None],
[white(Pawn), white(Pawn),   white(Pawn),   white(Pawn),  white(Pawn), white(Pawn),   white(Pawn),   white(Pawn)],
[white(Rook), white(Knight), white(Bishop), white(Queen), white(King), white(Bishop), white(Knight), white(Rook)]
            ],
            en_passant: None,
            castling_black: Castling::default(),
            castling_white: Castling::default()
        }
    }
}
impl Board {
    /// Create a new chess board with the default pieces
    pub fn new() -> Self {
        Self::default()
    }
    /// Get a reference to the piece at the requested position
    pub fn get(&self, pos: Pos) -> Option<Piece> {
        assert!(pos.is_valid());

        let Pos(x, y) = pos;
        self.pieces[y as usize][x as usize]
    }
    /// Get a mutable reference to the piece at the requested position
    pub fn get_mut(&mut self, pos: Pos) -> &mut Option<Piece> {
        assert!(pos.is_valid());

        let Pos(x, y) = pos;
        &mut self.pieces[y as usize][x as usize]
    }

    /// Return a reference over all the rows, starting at 8 going down to 1
    pub fn rows(&self) -> &[[Option<Piece>; WIDTH as usize]; WIDTH as usize] {
        &self.pieces
    }
    /// Return an iterator over all the rows, starting at 8 going down to 1
    pub fn iter(&self) -> std::slice::Iter<[Option<Piece>; WIDTH as usize]> {
        self.pieces.iter()
    }

    fn castling(&mut self, side: Side) -> &mut Castling {
        match side {
            Side::Black => &mut self.castling_black,
            Side::White => &mut self.castling_white
        }
    }

    /// Does extra validation for a move.
    /// Returns yes if the piece at `from` make the move `m`.
    /// Warning: This may return false positives if called on anything other than the results of `piece.moves()`!
    pub fn can_move(&mut self, from: Pos, m: Pos) -> bool {
        let Pos(from_x, from_y) = from;
        let Pos(rel_x, rel_y) = m;
        let dest = from + m;

        if !dest.is_valid() {
            return false;
        }
        let piece = match self.get(from) {
            Some(piece) => piece,
            None => return false
        };

        if self.get(dest).map(|p| piece.side == p.side).unwrap_or(false) {
            return false;
        }

        match piece.kind {
            PieceKind::Pawn =>
                // Capture diagonally, or en passant
                ((rel_x.abs() == 1) == (self.get(dest).is_some())
                    || (rel_x.abs() == 1 && self.en_passant == Some(Pos(from_x + rel_x, from_y))))
                // Optionally jump twice if at starting position, but don't jump over a piece
                && (rel_y.abs() != 2
                    || (from_y == edge_offset(piece.side, 1) && self.get(from + Pos(0, rel_y / 2)).is_none())),
            PieceKind::King => {
                if rel_x.abs() != 2 {
                    return true;
                }
                // Castling
                let row = edge_offset(piece.side, 0);
                if from_y != row {
                    return false;
                }
                let empty = if rel_x < 0 {
                    self.castling(piece.side).queenside
                        && self.get(Pos(0, row)) == Some(Piece { kind: PieceKind::Rook, side: piece.side })
                        && self.get(Pos(1, row)).is_none()
                        && self.get(Pos(2, row)).is_none()
                        && self.get(Pos(3, row)).is_none()
                } else {
                    self.castling(piece.side).kingside
                        && self.get(Pos(WIDTH-1, row)) == Some(Piece { kind: PieceKind::Rook, side: piece.side })
                        && self.get(Pos(WIDTH-2, row)).is_none()
                        && self.get(Pos(WIDTH-3, row)).is_none()
                };
                if !empty || self.check(piece.side).is_some() {
                    return false;
                }
                for i in 0..2 {
                    let pos = from + Pos(rel_x / (2-i), 0);
                    let undo = self.move_(from, pos);
                    let check = self.check(piece.side).is_some();
                    self.undo(undo);
                    if check {
                        return false;
                    }
                }
                true
            },
            _ => true
        }
    }
    /// Run the function `f` for each piece that belongs to `side`
    pub fn pieces(&self, side: Side) -> PieceIter {
        PieceIter {
            side,
            pos: Pos::default()
        }
    }
    /// Run the function `f` for each move that the piece at position `pos` can make.
    pub fn moves_for(&self, pos: Pos) -> MoveIter<impl FnMut(Pos) -> bool> {
        self.moves_for_filter(pos, |_| true)
    }
    /// Run the function `f` for each move that the piece at position `pos` can make.
    /// Allows filtering moves *before* they are validated.
    pub fn moves_for_filter<F>(&self, pos: Pos, filter: F) -> MoveIter<F>
        where F: FnMut(Pos) -> bool
    {
        let (moves, repeat) = match self.get(pos) {
            Some(piece) => piece.moves(),
            None => (ArrayVec::new(), false)
        };
        MoveIter {
            start: pos,
            filter,

            moves,
            repeat,

            repeat_cursor: None,
            i: 0
        }
    }

    /// Move a piece, replacing whatever was already on `to`. Handles any logic
    /// like spawning a queen. Can be undone.
    pub fn move_(&mut self, from: Pos, to: Pos) -> Change {
        let prev_en_passant = self.en_passant.take();

        let mut vec = ArrayVec::new();

        let piece = self.get_mut(from).take();
        let old = mem::replace(self.get_mut(to), piece);

        vec.extend(ArrayVec::from([
            Undo::Set(from, piece),
            Undo::Set(to, old)
        ]));

        if let Some(piece) = piece {
            let Pos(from_x, from_y) = from;
            let Pos(to_x, to_y) = to;
            match piece.kind {
                PieceKind::Pawn => {
                    let en_passant = Pos(to_x, from_y);
                    if to_y == edge_offset(!piece.side, 0) {
                        // Pawn moved all the way to the other's edge, let's upgrade it!
                        self.get_mut(to)
                            .as_mut()
                            .unwrap()
                            .kind = PieceKind::Queen;
                    } else if from_y == edge_offset(piece.side, 1) && to_y == edge_offset(piece.side, 3) {
                        // Did initial move, is subject to en passant
                        self.en_passant = Some(to);
                    } else if from_x != to_x && prev_en_passant == Some(en_passant) {
                        // Did en passant, kill victim
                        let killed = self.get_mut(en_passant).take();
                        vec.push(Undo::Set(en_passant, killed));
                    }
                },
                PieceKind::Rook => {
                    // Can no longer do a castling
                    let row = edge_offset(piece.side, 0);
                    let castling = self.castling(piece.side);

                    if from == Pos(0, row) {
                        if castling.queenside {
                            vec.push(Undo::Castling(piece.side, *castling));
                            castling.queenside = false;
                        }
                    } else if from == Pos(WIDTH-1, row) {
                        if castling.kingside {
                            vec.push(Undo::Castling(piece.side, *castling));
                            castling.kingside = false;
                        }
                    }
                },
                PieceKind::King => {
                    let castling = *self.castling(piece.side);

                    // Was this a castling?
                    let row = edge_offset(piece.side, 0);

                    let positions = if to_x - from_x == -2 && castling.queenside {
                        Some((
                            Pos(0, row),
                            Pos(to_x + 1, row)
                        ))
                    } else if to_x - from_x == 2 && castling.kingside {
                        Some((
                            Pos(WIDTH-1, row),
                            Pos(to_x - 1, row)
                        ))
                    } else {
                        None
                    };
                    if let Some((rook_from, rook_to)) = positions {
                        let piece = self.get_mut(rook_from).take();
                        *self.get_mut(rook_to) = piece;

                        vec.extend(ArrayVec::from([
                            Undo::Set(rook_from, piece),
                            Undo::Set(rook_to, None)
                        ]));
                    }

                    // You can no longer do a castling
                    if castling.kingside || castling.queenside {
                        let castling = self.castling(piece.side); // can't use mutable reference above

                        vec.push(Undo::Castling(piece.side, *castling));
                        *castling = Castling {
                            queenside: false,
                            kingside: false
                        };
                    }
                },
                _ => ()
            }

            if prev_en_passant.is_some() || self.en_passant.is_some() {
                vec.push(Undo::EnPassant(prev_en_passant));
            }
        }

        vec
    }
    /// Undo a move
    pub fn undo(&mut self, change: Change) {
        for undo in change {
            match undo {
                Undo::Set(pos, piece) => *self.get_mut(pos) = piece,
                Undo::EnPassant(pos) => self.en_passant = pos,
                Undo::Castling(side, castling) => *self.castling(side) = castling
            }
        }
    }

    /// Return whatever piece is threatening the specified side's king, if any
    pub fn check(&mut self, side: Side) -> Option<Pos> {
        let mut pieces = self.pieces(!side);
        while let Some(from) = pieces.next(self) {
            let piece = self.get(from);

            // Prevent infinite loop:
            // Castling detects if in check, check detects if can do castling
            let mut moves = self.moves_for_filter(from, |m| !is_castling(piece, m));

            while let Some(to) = moves.next(self) {
                if self.get(to) == Some(Piece { kind: PieceKind::King, side }) {
                    return Some(from);
                }
            }
        }

        None
    }
    /// Returns true if the specified side cannot make a move that's not in check
    pub fn is_checkmate(&mut self, side: Side) -> bool {
        let mut pieces = self.pieces(side);
        while let Some(from) = pieces.next(self) {
            let mut moves = self.moves_for(from);
            while let Some(to) = moves.next(self) {
                let undo = self.move_(from, to);
                let check = self.check(side);
                self.undo(undo);

                if check.is_none() {
                    return false;
                }
            }
        }
        true
    }
}
impl<'a> IntoIterator for &'a Board {
    type Item = &'a [Option<Piece>; WIDTH as usize];
    type IntoIter = std::slice::Iter<'a, [Option<Piece>; WIDTH as usize]>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// While `PieceIter` doesn't actually implement the `Iterator` trait, it does
/// let you iterate pieces using the `next` function
#[derive(Debug, Clone, Copy)]
pub struct PieceIter {
    side: Side,
    pos: Pos
}
impl PieceIter {
    /// Gets the next position of a piece in the "iterator"
    pub fn next(&mut self, board: &Board) -> Option<Pos> {
        if !self.pos.is_valid() {
            return None;
        }
        let mut pos;
        loop {
            pos = self.pos;
            if !pos.is_valid() {
                return None;
            }

            self.pos = pos.next();

            if board.get(pos).map(|p| p.side == self.side).unwrap_or(false) {
                break;
            }
        }
        Some(pos)
    }
}

/// While `MoveIter` doesn't actually implement the `Iterator` trait, it does
/// let you iterate moves using the `next` function
#[derive(Debug, Clone)]
pub struct MoveIter<F>
    where F: FnMut(Pos) -> bool
{
    start: Pos,
    filter: F,

    moves: ArrayVec<[Pos; 10]>,
    repeat: bool,

    repeat_cursor: Option<(Pos, Pos)>,
    i: usize
}
impl<F> MoveIter<F>
    where F: FnMut(Pos) -> bool
{
    /// Gets the next destination for a move in the "iterator"
    pub fn next(&mut self, board: &mut Board) -> Option<Pos> {
        if let Some((velocity, ref mut m)) = self.repeat_cursor {
            *m += velocity;
            if board.can_move(self.start, *m) {
                let target = self.start + *m;
                if board.get(target).is_some() {
                    self.repeat_cursor = None;
                }
                return Some(target);
            } else {
                self.repeat_cursor = None;
            }
        }
        while let Some(&m) = self.moves.get(self.i) {
            self.i += 1;
            if (self.filter)(m) && board.can_move(self.start, m) {
                let target = self.start + m;
                if self.repeat && board.get(target).is_none() {
                    self.repeat_cursor = Some((m, m));
                }
                return Some(target);
            }
        }
        None
    }
}
