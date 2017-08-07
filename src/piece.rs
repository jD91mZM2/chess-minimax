use *;
use std::str::FromStr;

pub struct NoSuchPieceErr;

#[derive(Debug, Clone, Copy)]
pub enum Piece {
	King(bool),
	Queen(bool),
	Rook(bool),
	Bishop(bool),
	Knight(bool),
	Pawn(bool),
	Empty
}

impl FromStr for Piece {
	type Err = NoSuchPieceErr;

	fn from_str(input: &str) -> Result<Self, Self::Err> {
		macro_rules! from_str {
			($mine:expr, $your:expr) => {
				Ok(match input {
					concat!($mine, "king") => Piece::King(true),
					concat!($your, "king") => Piece::King(false),
					concat!($mine, "queen") => Piece::Queen(true),
					concat!($your, "queen") => Piece::Queen(false),
					concat!($mine, "rook") => Piece::Rook(true),
					concat!($your, "rook") => Piece::Rook(false),
					concat!($mine, "bishop") => Piece::Bishop(true),
					concat!($your, "bishop") => Piece::Bishop(false),
					concat!($mine, "knight") => Piece::Knight(true),
					concat!($your, "knight") => Piece::Knight(false),
					concat!($mine, "pawn") => Piece::Pawn(true),
					concat!($your, "pawn") => Piece::Pawn(false),
					_ => return Err(NoSuchPieceErr),
				})
			}
		}
		#[cfg(not(feature = "white"))]
		return from_str!("black", "white");
		#[cfg(feature = "white")]
		from_str!("white", "black")
	}
}
impl Piece {
	pub fn to_char(&self) -> char {
		macro_rules! to_char {
			($i_am_black:expr, $i_am_white:expr) => {
				match *self {
					Piece::King($i_am_white) => '♚',
					Piece::King($i_am_black) => '♔',
					Piece::Queen($i_am_white) => '♛',
					Piece::Queen($i_am_black) => '♕',
					Piece::Rook($i_am_white) => '♜',
					Piece::Rook($i_am_black) => '♖',
					Piece::Bishop($i_am_white) => '♝',
					Piece::Bishop($i_am_black) => '♗',
					Piece::Knight($i_am_white) => '♞',
					Piece::Knight($i_am_black) => '♘',
					Piece::Pawn($i_am_white) => '♟',
					Piece::Pawn($i_am_black) => '♙',
					Piece::Empty => ' ',
				}
			}
		}
		#[cfg(not(feature = "white"))]
		return to_char!(true, false);
		#[cfg(feature = "white")]
		to_char!(false, true)
	}

	pub fn is_empty(&self) -> bool {
		match *self {
			Piece::Empty => true,
			_ => false,
		}
	}
	pub fn is_mine(&self) -> bool {
		match *self {
			Piece::King(mine) |
			Piece::Queen(mine) |
			Piece::Rook(mine) |
			Piece::Bishop(mine) |
			Piece::Knight(mine) |
			Piece::Pawn(mine) => mine,
			Piece::Empty => false,
		}
	}

	pub fn recursive(&self) -> bool {
		match *self {
			Piece::King(_) |
			Piece::Knight(_) |
			Piece::Pawn(_) |
			Piece::Empty => false,

			Piece::Queen(_) |
			Piece::Rook(_) |
			Piece::Bishop(_) => true,
		}
	}

	pub fn moves(&self) -> Vec<(i8, i8)> {
		// Returns DownRight
		match *self {
			Piece::King(_) |
			Piece::Queen(_) => vec![(0, 1), (1, 1)],
			Piece::Rook(_) => vec![(0, 1)],
			Piece::Bishop(_) => vec![(1, 1)],
			Piece::Knight(_) => vec![(1, 2)],
			Piece::Pawn(_) => vec![(0, 1), (1, 1), (0, 2)],
			Piece::Empty => vec![],
		}
	}

	pub fn worth(&self) -> u8 {
		// Returns DownRight
		match *self {
			Piece::King(_) => 100,
			Piece::Queen(_) => 9,
			Piece::Rook(_) => 5,
			Piece::Bishop(_) |
			Piece::Knight(_) => 3,
			Piece::Pawn(_) => 1,
			Piece::Empty => 0,
		}
	}

	pub fn can_move(&self, board: &Board, rel: (i8, i8), abs: (i8, i8)) -> bool {
		let (rel_x, rel_y) = rel;
		let (x, y) = abs;

		if self.is_empty() {
			return false;
		}
		if x < 0 || x >= 8 || y < 0 || y >= 8 {
			return false;
		}

		let piece = board_get(board, abs);
		if !piece.is_empty() && self.is_mine() == piece.is_mine() {
			return false;
		}

		match *self {
			Piece::Pawn(mine) => {
				(mine == (rel_y < 0)) &&
					(rel_x.abs() != 2) &&
					(rel_y.abs() != 2 ||
						(((!mine && y == 3) ||
						(mine && y == 4)) &&
						((rel_y == 2 && board_get(board, (x, y - 1)).is_empty()) ||
						 (rel_y == -2 && board_get(board, (x, y + 1)).is_empty())))) &&
					((rel_x.abs() == 0) == piece.is_empty()) &&
					(rel_x.abs() != 1 || rel_y.abs() == 1)
			},
			_ => true,
		}
	}

	pub fn possible_moves(&self, board: &Board, abs: (i8, i8)) -> Vec<(i8, i8)> {
		// This is the most called function according to a profiler.
		// I'm willing to pre-allocate too much if that means less allocations.

		let (x, y) = abs;
		let recursive = self.recursive();

		let moves = self.moves();
		let mut possible_moves = Vec::with_capacity(moves.len() * 4);

		for m in moves {
			for direction in &DIRECTIONS_ALL {
				let (rel_x, rel_y) = rotate(m, direction);
				let (mut new_x, mut new_y) = (rel_x, rel_y);

				loop {
					let abs = (x + new_x, y + new_y);

					if possible_moves.contains(&abs) {
					} else if self.can_move(board, (new_x, new_y), abs) {
						possible_moves.push(abs);
					} else {
						break;
					}

					if !recursive || !board_get(board, abs).is_empty() {
						break;
					}

					new_x += rel_x;
					new_y += rel_y;

				}
			}
		}

		possible_moves
	}
}
