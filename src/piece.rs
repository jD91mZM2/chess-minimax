use *;

pub enum Piece {
	King(bool),
	Queen(bool),
	Rook(bool),
	Bishop(bool),
	Knight(bool),
	Pawn(bool),
	Empty
}

impl Piece {
	pub fn to_char(&self) -> char {
		match *self {
			Piece::King(true) => '♚',
			Piece::King(false) => '♔',
			Piece::Queen(true) => '♛',
			Piece::Queen(false) => '♕',
			Piece::Rook(true) => '♜',
			Piece::Rook(false) => '♖',
			Piece::Bishop(true) => '♝',
			Piece::Bishop(false) => '♗',
			Piece::Knight(true) => '♞',
			Piece::Knight(false) => '♘',
			Piece::Pawn(true) => '♟',
			Piece::Pawn(false) => '♙',
			Piece::Empty => ' ',
		}
	}

	pub fn is_empty(&self) -> bool {
		match *self {
			Piece::Empty => true,
			_ => false,
		}
	}

	pub fn recursive(&self) -> bool {
		match *self {
			Piece::King(_) => false,
			Piece::Queen(_) => true,
			Piece::Rook(_) => true,
			Piece::Bishop(_) => true,
			Piece::Knight(_) => false,
			Piece::Pawn(_) => false,
			Piece::Empty => false,
		}
	}

	pub fn moves(&self) -> Vec<(i8, i8)> {
		// Returns DownRight
		match *self {
			Piece::King(_) => vec![(0, 1), (1, 1)],
			Piece::Queen(_) => vec![(0, 1), (1, 1)],
			Piece::Rook(_) => vec![(0, 1)],
			Piece::Bishop(_) => vec![(1, 1)],
			Piece::Knight(_) => vec![(1, 2)],
			Piece::Pawn(_) => vec![(0, 1), (1, 1)],
			Piece::Empty => vec![],
		}
	}

	pub fn can_move(&self, board: Board, rel: (i8, i8), abs: (i8, i8)) -> bool {
		let (rel_x, rel_y) = rel;
		let (x, y) = abs;

		if x < 0 || x >= 8 || y < 0 || y >= 8 {
			return false;
		}

		match *self {
			Piece::Pawn(mine) => {
				(!mine || rel_y == 1) &&
					(mine || rel_y == -1) &&
					(rel_x.abs() != 0 || board[y as usize][x as usize].is_empty()) &&
					(rel_x.abs() == 0 || !board[y as usize][x as usize].is_empty())
			},
			_ => true,
		}
	}
}
