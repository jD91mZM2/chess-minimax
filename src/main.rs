mod board;
mod piece;

pub use board::*;
pub use piece::*;

#[derive(Debug)]
pub enum Direction {
	UpLeft,
	UpRight,
	LeftUp,
	LeftDown,
	RightUp,
	RightDown,
	DownLeft,
	DownRight,
}

impl Direction {
	pub fn all() -> [Direction; 4] {
		[Direction::UpLeft, Direction::UpRight, Direction::DownLeft, Direction::DownRight]
	}
}

pub fn rotate(rel: (i8, i8), direction: &Direction) -> (i8, i8) {
	// Assumes current rotation is DownRight

	let (x, y) = rel;
	match *direction {
		Direction::UpLeft => (-x, -y),
		Direction::UpRight => (x, -y),
		Direction::LeftUp => (-y, -x),
		Direction::LeftDown => (y, -x),
		Direction::RightUp => (-y, x),
		Direction::RightDown => (y, x),
		Direction::DownLeft => (-x, y),
		Direction::DownRight => (x, y),
	}
}

fn main() {
	let board = board::make_board();

	let (x, y) = (0, 1);
	println!("{:?}", board[y as usize][x as usize].possible_moves(&board, (x, y)));
	println!("{}", board::board_string(&board));
}
