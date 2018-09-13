use crate::{
    board::{self, Board},
    piece::{Piece, PieceKind},
    Side
};
use std::io::{self, Read, Write};

/// Serialize a board to an I/O stream
pub fn serialize_board<W: Write>(out: &mut W, board: &Board) -> io::Result<()> {
    for row in board {
        for pieces in row.chunks(2) {
            let mut pieces_array = [None; 2];
            pieces_array.copy_from_slice(pieces);
            let byte = serialize_pieces(&pieces_array);
            out.write_all(&[byte])?;
        }
    }
    Ok(())
}
/// Deserialize a board from an I/O stream
pub fn deserialize_board<R: Read>(input: &mut R) -> io::Result<Board> {
    let mut pieces = [[None; board::WIDTH as usize]; board::WIDTH as usize];
    for y in 0..board::WIDTH as usize {
        let mut bytes = [0; board::WIDTH as usize / 2];
        input.read_exact(&mut bytes)?;

        let mut x = 0;
        for byte in &bytes {
            let p = deserialize_pieces(*byte);
            pieces[y][x..x+2].copy_from_slice(&p);
            x += 2;
        }
    }
    Ok(Board {
        pieces,
        en_passant: None // TODO!!!!
    })
}

/// Serialize 2 pieces into one single byte
pub fn serialize_pieces(pieces: &[Option<Piece>; 2]) -> u8 {
    let mut byte = 0;
    for piece in pieces {
        byte <<= 3;
        byte += match piece.map(|p| p.kind) {
            None => 0,
            Some(PieceKind::Pawn) => 1,
            Some(PieceKind::Knight) => 2,
            Some(PieceKind::Bishop) => 3,
            Some(PieceKind::Rook) => 4,
            Some(PieceKind::Queen) => 5,
            Some(PieceKind::King) => 6
        };
        byte <<= 1;
        if piece.map(|p| p.side == Side::White).unwrap_or(false) {
            byte += 1;
        }
    }
    byte
}
/// Deserialize a byte into 2 pieces
pub fn deserialize_pieces(byte: u8) -> [Option<Piece>; 2] {
    let mut pieces = [None; 2];
    for i in 0..2 {
        let mut byte = byte >> ((1 - i) * 4);
        let side = if byte & 1 == 1 {
            Side::White
        } else {
            Side::Black
        };
        byte >>= 1;
        pieces[i] = match byte & 0b111 {
            1 => Some(Piece { kind: PieceKind::Pawn, side }),
            2 => Some(Piece { kind: PieceKind::Knight, side }),
            3 => Some(Piece { kind: PieceKind::Bishop, side }),
            4 => Some(Piece { kind: PieceKind::Rook, side }),
            5 => Some(Piece { kind: PieceKind::Queen, side }),
            6 => Some(Piece { kind: PieceKind::King, side }),
            // Zero or invalid
            _ => None
        };
    }
    pieces
}
