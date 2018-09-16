use arrayvec::ArrayVec;
use crate::{
    Pos,
    Side
};
use std::fmt;

/// A chess piece on the board, a kind of piece and what side it is on
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub side: Side
}
impl Piece {
    /// Convenience function to create a black piece
    pub fn black(kind: PieceKind) -> Self {
        Self {
            kind,
            side: Side::Black
        }
    }
    /// Convenience function to create a white piece
    pub fn white(kind: PieceKind) -> Self {
        Self {
            kind,
            side: Side::White
        }
    }

    /// Get a unicode character for this piece
    pub fn to_char(&self) -> char {
        match (self.side, self.kind) {
            (Side::Black, PieceKind::Pawn) => '♟',
            (Side::Black, PieceKind::Knight) => '♞',
            (Side::Black, PieceKind::Bishop) => '♝',
            (Side::Black, PieceKind::Rook) => '♜',
            (Side::Black, PieceKind::Queen) => '♛',
            (Side::Black, PieceKind::King) => '♚',

            (Side::White, PieceKind::Pawn) => '♙',
            (Side::White, PieceKind::Knight) => '♘',
            (Side::White, PieceKind::Bishop) => '♗',
            (Side::White, PieceKind::Rook) => '♖',
            (Side::White, PieceKind::Queen) => '♕',
            (Side::White, PieceKind::King) => '♔'
        }
    }

    /// All moves this piece can make.
    /// Note: Some moves may or may not be possible, depending on the position on the board.
    pub fn moves(&self) -> (ArrayVec<[Pos; 10]>, bool) {
        let mut vec = ArrayVec::new();

        const ROOK_MOVES: [Pos; 4] = [
            Pos(0, 1),
            Pos(0, -1),
            Pos(1, 0),
            Pos(-1, 0)
        ];
        const BISHOP_MOVES: [Pos; 4] = [
            Pos(1, 1),
            Pos(1, -1),
            Pos(-1, 1),
            Pos(-1, -1)
        ];

        let repeat = match (self.side, self.kind) {
            (Side::Black, PieceKind::Pawn) => { vec.extend(ArrayVec::from([
                Pos(0, 1),
                Pos(0, 2),
                Pos(1, 1),
                Pos(-1, 1)
            ])); false },
            (Side::White, PieceKind::Pawn) => { vec.extend(ArrayVec::from([
                Pos(0, -1),
                Pos(0, -2),
                Pos(1, -1),
                Pos(-1, -1)
            ])); false },
            (_, PieceKind::Knight) => { vec.extend(ArrayVec::from([
                Pos(1, 2),
                Pos(1, -2),
                Pos(-1, 2),
                Pos(-1, -2),
                Pos(2, 1),
                Pos(2, -1),
                Pos(-2, 1),
                Pos(-2, -1)
            ])); false },
            (_, PieceKind::Bishop) => { vec.extend(BISHOP_MOVES.iter().cloned()); true },
            (_, PieceKind::Rook) => { vec.extend(ROOK_MOVES.iter().cloned()); true },
            (_, PieceKind::Queen)
            | (_, PieceKind::King) => {
                let mut moves = [Pos::default(); 8];
                moves[0..4].copy_from_slice(&ROOK_MOVES);
                moves[4..8].copy_from_slice(&BISHOP_MOVES);
                vec.extend(ArrayVec::from(moves));

                if self.kind == PieceKind::King {
                    // Castling
                    vec.extend(ArrayVec::from([
                        Pos(2, 0),
                        Pos(-2, 0),
                    ]));
                }

                self.kind == PieceKind::Queen
            },
        };
        (vec, repeat)
    }
}
impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}
/// A kind of chess piece
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}
impl PieceKind {
    /// Return the piece worth, according to
    /// https://en.wikipedia.org/wiki/Chess_piece_relative_value
    /// Note: The king's worth returns 0, because the minimax algorithm handles
    /// it separately.
    pub fn worth(self) -> u8 {
        match self {
            PieceKind::Pawn => 1,
            PieceKind::Knight
            | PieceKind::Bishop => 3,
            PieceKind::Rook => 5,
            PieceKind::Queen => 9,

            PieceKind::King => 0
        }
    }
}
