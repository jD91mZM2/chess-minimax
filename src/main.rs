mod board;

pub enum Part {
	King(bool),
	Queen(bool),
	Rook(bool),
	Bishop(bool),
	Knight(bool),
	Pawn(bool),
	Empty
}

impl Part {
	fn to_char(&self) -> char {
		match *self {
			Part::King(true) => '♚',
			Part::King(false) => '♔',
			Part::Queen(true) => '♛',
			Part::Queen(false) => '♕',
			Part::Rook(true) => '♜',
			Part::Rook(false) => '♖',
			Part::Bishop(true) => '♝',
			Part::Bishop(false) => '♗',
			Part::Knight(true) => '♞',
			Part::Knight(false) => '♘',
			Part::Pawn(true) => '♟',
			Part::Pawn(false) => '♙',
			Part::Empty => ' ',
		}
	}
}

fn main() {
	let board = board::make_board();

	println!("{}", board::board_string(&board));
}
