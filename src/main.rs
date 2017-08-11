#[cfg(feature = "websocket")]
extern crate websocket;
#[cfg(feature = "cpuprofiler")]
extern crate cpuprofiler;
#[cfg(feature = "cpuprofiler")]
use cpuprofiler::PROFILER;

mod board;
mod piece;
mod search;
#[cfg(not(feature = "websocket"))]
mod input;
#[cfg(feature = "websocket")]
mod ws;

pub use board::*;
pub use piece::*;
pub use search::*;

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

const DIRECTIONS_ALL: [Direction; 8] = [
	Direction::UpLeft,
	Direction::UpRight,
	Direction::LeftUp,
	Direction::LeftDown,
	Direction::RightUp,
	Direction::RightDown,
	Direction::DownLeft,
	Direction::DownRight
];


fn rotate(rel: (i8, i8), direction: &Direction) -> (i8, i8) {
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
	#[cfg(all(feature = "websocket", feature = "cpuprofiler"))]
	panic!("Oh no, you can't have both websocket and cpuprofiler");

	#[cfg(not(feature = "websocket"))]
	input::main();
	#[cfg(feature = "websocket")]
	ws::main();
}
