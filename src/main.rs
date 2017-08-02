mod board;
mod piece;

pub use board::*;
pub use piece::*;

pub enum Direction {
	UpLeft,
	UpRight,
	DownLeft,
	DownRight,
}

pub fn rotate(rel: (i8, i8), direction: Direction) -> (i8, i8) {
	// Assumes current rotation is DownRight

	let (x, y) = rel;
	match direction {
		Direction::UpLeft => (-x, -y),
		Direction::UpRight => (x, -y),
		Direction::DownLeft => (-x, y),
		Direction::DownRight => (x, y),
	}
}

fn main() {
	let board = board::make_board();

	println!("{}", board::board_string(&board));
}
