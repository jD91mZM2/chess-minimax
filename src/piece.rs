use ::*;
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
		Ok(match input {
			"blackking" => Piece::King(true),
			"whiteking" => Piece::King(false),
			"blackqueen" => Piece::Queen(true),
			"whitequeen" => Piece::Queen(false),
			"blackrook" => Piece::Rook(true),
			"whiterook" => Piece::Rook(false),
			"blackbishop" => Piece::Bishop(true),
			"whitebishop" => Piece::Bishop(false),
			"blackknight" => Piece::Knight(true),
			"whiteknight" => Piece::Knight(false),
			"blackpawn" => Piece::Pawn(true),
			"whitepawn" => Piece::Pawn(false),
			_ => return Err(NoSuchPieceErr),
		})
	}
}
impl Piece {
	pub fn to_char(&self) -> char {
		match *self {
			Piece::King(false) => '♚',
			Piece::King(true) => '♔',
			Piece::Queen(false) => '♛',
			Piece::Queen(true) => '♕',
			Piece::Rook(false) => '♜',
			Piece::Rook(true) => '♖',
			Piece::Bishop(false) => '♝',
			Piece::Bishop(true) => '♗',
			Piece::Knight(false) => '♞',
			Piece::Knight(true) => '♘',
			Piece::Pawn(false) => '♟',
			Piece::Pawn(true) => '♙',
			Piece::Empty => ' ',
		}
	}

	pub fn is_empty(&self) -> bool {
		match *self {
			Piece::Empty => true,
			_ => false,
		}
	}
	pub fn is_black(&self) -> bool {
		match *self {
			Piece::King(black) |
			Piece::Queen(black) |
			Piece::Rook(black) |
			Piece::Bishop(black) |
			Piece::Knight(black) |
			Piece::Pawn(black) => black,
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
		if !piece.is_empty() && self.is_black() == piece.is_black() {
			return false;
		}

		match *self {
			Piece::Pawn(black) => {
				(black == (rel_y < 0)) &&
					(rel_x.abs() != 2) &&
					(rel_y.abs() != 2 ||
						(((!black && y == 3) ||
						(black && y == 4)) &&
						(rel_y == 2 && board_get(board, (x, y - 1)).is_empty() ||
						 rel_y == -2 && board_get(board, (x, y + 1)).is_empty()))) &&
					((rel_x.abs() == 0) == piece.is_empty())
			},
			_ => true,
		}
	}

	pub fn possible_moves(&self, board: &Board, abs: (i8, i8)) -> Vec<(i8, i8)> {
		let (x, y) = abs;
		let recursive = self.recursive();

		let mut moves = Vec::new();

		for m in self.moves() {
			for direction in &Direction::all() {
				let (rel_x, rel_y) = rotate(m, direction);
				let (mut new_x, mut new_y) = (rel_x, rel_y);

				loop {
					let abs = (x + new_x, y + new_y);

					if moves.contains(&abs) {
					} else if self.can_move(board, (new_x, new_y), abs) {
						moves.push(abs);
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

		moves
	}
}
