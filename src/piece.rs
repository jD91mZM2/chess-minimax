use *;
use std::str::FromStr;

pub struct NoSuchPieceErr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
	#[cfg(feature = "cache")]
	pub fn to_byte(&self) -> u8 {
		let (mine, mut byte) = match *self {
			Piece::King(mine) => (mine, 14),
			Piece::Queen(mine) => (mine, 12),
			Piece::Rook(mine) => (mine, 10),
			Piece::Bishop(mine) => (mine, 8),
			Piece::Knight(mine) => (mine, 6),
			Piece::Pawn(mine) => (mine, 4),
			Piece::Empty => (false, 2),
		};
		// Can't use 0-indexed because null bytes.

		if !mine {
			byte += 1;
		}

		byte
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

	pub fn moves(&self) -> [Option<Pos>; 3] {
		// Returns DownRight
		match *self {
			Piece::King(_) |
			Piece::Queen(_) =>  [ Some((0, 1)), Some((1, 1)), None         ],
			Piece::Rook(_) =>   [ Some((0, 1)), None,         None         ],
			Piece::Bishop(_) => [ Some((1, 1)), None,         None         ],
			Piece::Knight(_) => [ Some((1, 2)), None,         None         ],
			Piece::Pawn(_) =>   [ Some((0, 1)), Some((1, 1)), Some((0, 2)) ],
			Piece::Empty =>     [ None,         None,         None         ],
		}
	}

	pub fn worth(&self) -> u8 {
		match *self {
			Piece::King(_) => 0, // Handled separately
			Piece::Queen(_) => 9,
			Piece::Rook(_) => 5,
			Piece::Bishop(_) |
			Piece::Knight(_) => 3,
			Piece::Pawn(_) => 1,
			Piece::Empty => 0,
		}
	}

	pub fn can_move(&self, board: &Board, rel: Pos, abs: Pos) -> bool {
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
				mine == (rel_y < 0) &&  // Only move forwards
					(rel_x.abs() != 2) &&
					(rel_y.abs() != 2 || // Allow jumping twice initially (but don't jump over a piece)
						(((!mine && y == 3) ||
						(mine && y == 4)) &&
						((rel_y == 2 && board_get(board, (x, y - 1)).is_empty()) ||
						 (rel_y == -2 && board_get(board, (x, y + 1)).is_empty())))) &&
					(rel_x.abs() != 1 || rel_y.abs() == 1) && // Capture diagonally
					(rel_x.abs() != 0) == (!piece.is_empty() || en_passant_get_capture(board, mine, abs).is_some())
			},
			_ => true,
		}
	}

	pub fn possible_moves(&self, board: &Board, abs: Pos) -> Vec<Pos> {
		// This is the most called function according to a profiler.
		// I'm willing to pre-allocate too much if that means less allocations.

		let (x, y) = abs;
		let recursive = self.recursive();

		let moves = self.moves();
		// moves is max 3.
		// DIRECTIONS_ALL is always 8.
		// This pre-allocation does not account for recursive moves.
		// 3 * 8 = 24
		let mut possible_moves = Vec::with_capacity(24);

		for m in &moves {
			let m = match *m {
				Some(m) => m,
				None => continue,
			};
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
