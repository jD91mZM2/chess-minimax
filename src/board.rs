use crate::{
    Pos,
    Side,
    piece::{Piece, PieceKind}
};

/// The width (and height, because square) of the board
pub const WIDTH: i8 = 8;

/// A typical chess board
pub struct Board {
    side: Side,
    pieces: [[Option<Piece>; WIDTH as usize]; WIDTH as usize],
}
impl Board {
    /// Create a new chess board with the default pieces
    pub fn new(side: Side) -> Self {
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
            side,
            pieces: [
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
    /// Get a reference to the piece at the requested position
    pub fn get(&self, pos: Pos) -> Option<&Piece> {
        assert!(pos.is_valid());

        let Pos(x, y) = pos;
        self.pieces[y as usize][x as usize].as_ref()
    }
    /// Get a mutable reference to the piece at the requested position
    pub fn get_mut(&mut self, pos: Pos) -> Option<&mut Piece> {
        assert!(pos.is_valid());

        let Pos(x, y) = pos;
        self.pieces[y as usize][x as usize].as_mut()
    }

    /// Returns the side the computer plays as
    pub fn side(&self) -> Side {
        self.side
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
    pub fn each_piece<F, T>(&mut self, side: Side, mut f: F) -> Option<T>
        where F: FnMut(&mut Self, Pos) -> Option<T>
    {
        for y in 0..WIDTH {
            for x in 0..WIDTH {
                let pos = Pos(x, y);
                if self.get(pos).map(|p| p.side != side).unwrap_or(true) {
                    continue;
                }
                if let ret @ Some(_) = f(self, pos) {
                    return ret;
                }
            }
        }
        None
    }
    pub fn each_move_for<F, T>(&mut self, pos: Pos, mut f: F) -> Option<T>
        where F: FnMut(&mut Self, Pos) -> Option<T>
    {
        let piece = match self.get(pos) {
            Some(piece) => piece,
            None => return None
        };

        let (moves, repeat) = piece.moves();
        for &original_move in &moves {
            let mut m = original_move;
            loop {
                if !self.can_move(pos, m) {
                    break;
                }

                if let ret @ Some(_) = f(self, pos + m) {
                    return ret;
                }

                if !repeat {
                    break
                }

                m += original_move;
            }
        }
        None
    }
}
impl<'a> IntoIterator for &'a Board {
    type Item = &'a [Option<Piece>; WIDTH as usize];
    type IntoIter = std::slice::Iter<'a, [Option<Piece>; WIDTH as usize]>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
