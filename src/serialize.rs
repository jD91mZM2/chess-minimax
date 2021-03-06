use crate::{
    board::{self, Board, Castling},
    piece::{Piece, PieceKind},
    Pos,
    Side
};
use std::io::{self, Read, Write};

const HAS_EN_PASSANT: u8 = 1;
const BLACK_CASTLING_QUEENSIDE: u8 = 1 << 1;
const BLACK_CASTLING_KINGSIDE:  u8 = 1 << 2;
const WHITE_CASTLING_QUEENSIDE: u8 = 1 << 3;
const WHITE_CASTLING_KINGSIDE:  u8 = 1 << 4;

/// Serialize a board to an I/O stream
pub fn serialize_board<W: Write>(out: &mut W, board: &Board) -> io::Result<()> {
    let mut flags = 0;
    if board.en_passant.is_some() {
        flags |= HAS_EN_PASSANT;
    }
    if board.castling_black.queenside { flags |= BLACK_CASTLING_QUEENSIDE; }
    if board.castling_black.kingside { flags |= BLACK_CASTLING_KINGSIDE; }
    if board.castling_white.queenside { flags |= WHITE_CASTLING_QUEENSIDE; }
    if board.castling_white.kingside { flags |= WHITE_CASTLING_KINGSIDE; }
    out.write_all(&[flags])?;

    if let Some(en_passant) = board.en_passant {
        out.write_all(&[serialize_pos(en_passant)])?;
    }

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
    let mut byte = [0];
    input.read_exact(&mut byte)?;
    let flags = byte[0];

    let en_passant = if flags & HAS_EN_PASSANT == HAS_EN_PASSANT {
        input.read_exact(&mut byte)?;
        deserialize_pos(byte[0])
    } else {
        None
    };

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
        en_passant,
        castling_black: Castling {
            queenside: flags & BLACK_CASTLING_QUEENSIDE == BLACK_CASTLING_QUEENSIDE,
            kingside:  flags & BLACK_CASTLING_KINGSIDE  == BLACK_CASTLING_KINGSIDE
        },
        castling_white: Castling {
            queenside: flags & WHITE_CASTLING_QUEENSIDE == WHITE_CASTLING_QUEENSIDE,
            kingside:  flags & WHITE_CASTLING_KINGSIDE  == WHITE_CASTLING_KINGSIDE
        }
    })
}

/// Serialize a position into a byte
pub fn serialize_pos(pos: Pos) -> u8 {
    assert!(pos.is_valid());
    let Pos(x, y) = pos;
    x as u8 + y as u8 * board::WIDTH as u8
}
/// Deserialize a byte into a position
pub fn deserialize_pos(byte: u8) -> Option<Pos> {
    Some(Pos(byte as i8 % board::WIDTH as i8, byte as i8 / board::WIDTH as i8))
        .filter(|pos| pos.is_valid())
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
