use crate::{
    Pos,
    Side,
    piece::{Piece, PieceKind},
    utils::stackvec::StackVec
};
use std::mem;

pub type Change = StackVec<[Undo; 3]>;

/// Describes how to revert a change to the board. Used for undoing moves.
pub enum Undo {
    Set(Pos, Option<Piece>)
    // TODO
}

/// The width (and height, because square) of the board
pub const WIDTH: i8 = 8;

/// A typical chess board
#[derive(Debug, Clone)]
pub struct Board {
    pub(crate) pieces: [[Option<Piece>; WIDTH as usize]; WIDTH as usize],
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
//[black(King), None,          None,          None,         None,        None,          None,          None],
//[None,        None,          None,          None,         None,        None,          None,          None],
//[None,        None,          None,          None,         None,        None,          None,          None],
//[None,        None,          None,          None,         white(Rook), None,          white(Rook),   None],
//[None,        None,          None,          None,         None,        None,          None,          None],
//[None,        None,          None,          None,         None,        None,          None,          None],
//[None,        None,          None,          None,         None,        None,          None,          None],
//[None,        None,          None,          None,         None,        None,          None,          None],
[black(Rook), black(Knight), black(Bishop), black(Queen), black(King), black(Bishop), black(Knight), black(Rook)],
[black(Pawn), black(Pawn),   black(Pawn),   black(Pawn),  black(Pawn), black(Pawn),   black(Pawn),   black(Pawn)],
[None,        None,          None,          None,         None,        None,          None,          None],
[None,        None,          None,          None,         None,        None,          None,          None],
[None,        None,          None,          None,         None,        None,          None,          None],
[None,        None,          None,          None,         None,        None,          None,          None],
[white(Pawn), white(Pawn),   white(Pawn),   white(Pawn),  white(Pawn), white(Pawn),   white(Pawn),   white(Pawn)],
[white(Rook), white(Knight), white(Bishop), white(Queen), white(King), white(Bishop), white(Knight), white(Rook)]
            ]
        }
    }
}
impl Board {
    /// Create a new chess board with the default pieces
    pub fn new() -> Self {
        Self::default()
    }
    /// Get a reference to the piece at the requested position
    pub fn get(&self, pos: Pos) -> Option<&Piece> {
        assert!(pos.is_valid());

        let Pos(x, y) = pos;
        self.pieces[y as usize][x as usize].as_ref()
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

    /// Does extra validation for a move.
    /// Returns yes if the piece at `from` make the move `m`.
    /// Warning: This may return false positives if called on anything other than the results of `piece.moves()`!
    pub fn can_move(&self, from: Pos, m: Pos) -> bool {
        let Pos(_from_x, from_y) = from;
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
                // Capture diagonally
                (rel_x.abs() == 1) == (self.get(dest).is_some())
                // Optionally jump twice if at starting position, but don't jump over a piece
                && (rel_y.abs() != 2
                    || (piece.side == Side::White && from_y == WIDTH-2 && self.get(from + Pos(0, rel_y / 2)).is_none())
                    || (piece.side == Side::Black && from_y == 1 && self.get(from + Pos(0, rel_y / 2)).is_none())),
            _ => true
        }
    }
    /// Run the function `f` for each piece that belongs to `side`
    pub fn pieces(&mut self, side: Side) -> PieceIter {
        PieceIter {
            side,
            pos: Pos::default()
        }
    }
    /// Run the function `f` for each move that the piece at position `pos` can make
    pub fn moves_for(&mut self, pos: Pos) -> MoveIter {
        let (moves, repeat) = match self.get(pos) {
            Some(piece) => piece.moves(),
            None => (StackVec::new(), false)
        };
        MoveIter {
            start: pos,
            repeat,
            repeat_cursor: None,
            moves,
            i: 0
        }
    }

    /// Move a piece, replacing whatever was already on `to`. Handles any logic
    /// like spawning a queen. Can be undone.
    pub fn move_(&mut self, from: Pos, to: Pos) -> Change {
        let mut vec = StackVec::new();

        let piece = mem::replace(self.get_mut(from), None);
        let old = mem::replace(self.get_mut(to), piece);

        vec.append([
            Undo::Set(from, piece),
            Undo::Set(to, old)
        ]);
        vec
    }
    /// Undo a move
    pub fn undo(&mut self, change: Change) {
        for undo in change {
            match undo {
                Undo::Set(pos, piece) => *self.get_mut(pos) = piece
            }
        }
    }

    /// Calculate the total score for a certain side
    pub fn score(&mut self, side: Side) -> i16 {
        let mut score = 0;
        let mut pieces = self.pieces(side);
        while let Some(pos) = pieces.next(&self) {
            let piece = self.get(pos).unwrap();
            score += piece.kind.worth() as i16;
        }
        score
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
pub struct MoveIter {
    start: Pos,
    repeat: bool,
    repeat_cursor: Option<(Pos, Pos)>,
    moves: StackVec<[Pos; 8]>,
    i: usize
}
impl MoveIter {
    /// Gets the next destination for a move in the "iterator"
    pub fn next(&mut self, board: &Board) -> Option<Pos> {
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
            if board.can_move(self.start, m) {
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
