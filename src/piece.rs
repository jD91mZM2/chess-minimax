use *;

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
			Piece::King(black) => black,
			Piece::Queen(black) => black,
			Piece::Rook(black) => black,
			Piece::Bishop(black) => black,
			Piece::Knight(black) => black,
			Piece::Pawn(black) => black,
			Piece::Empty => false,
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
			Piece::Pawn(_) => vec![(0, 1), (1, 1), (0, 2)],
			Piece::Empty => vec![],
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


		let piece = &board[y as usize][x as usize];
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
						(rel_y == 2 && board[y as usize - 1][x as usize].is_empty() ||
						 rel_y == -2 && board[y as usize + 1][x as usize].is_empty()))) &&
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

					if !recursive || !board[abs.1 as usize][abs.0 as usize].is_empty() {
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
