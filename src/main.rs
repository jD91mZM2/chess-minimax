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

fn position_string(input: (i8, i8), output: &mut String) {
	output.push(std::char::from_u32((7 - input.0) as u32 + 'A' as u32).unwrap());
	output.push(std::char::from_u32(input.1 as u32 + '1' as u32).unwrap());
}
fn positions_join<'a, I: Iterator<Item = (i8, i8)>>(input: I) -> String {
	let mut output = String::new();
	let mut first  = true;

	for pos in input {
		if first {
			first = false
		} else {
			output.push_str(", ");
		}

		position_string(pos, &mut output)
	}

	output
}
fn parse_position(input: &str) -> Option<(i8, i8)> {
	let (mut x, mut y) = (0, 0);

	for c in input.chars() {
		let code = c as u32;

		if code >= 'a' as u32 && code <= 'h' as u32 {
			x = (7 - (code - 'a' as u32)) as i8
		} else if code >= 'A' as u32 && code <= 'H' as u32 {
			x = (7 - (code - 'A' as u32)) as i8
		} else if code >= '1' as u32 && code <= '8' as u32 {
			y = (code - '1' as u32) as i8
		} else {
			return None;
		}
	}

	Some((x, y))
}

fn main() {
	#[cfg(not(feature = "websocket"))]
	input::main();
	#[cfg(feature = "websocket")]
	ws::main();
}
