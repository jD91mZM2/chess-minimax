#[cfg(feature = "cpuprofiler")]
extern crate cpuprofiler;
#[cfg(feature = "cpuprofiler")]
use cpuprofiler::PROFILER;

mod board;
mod piece;
mod search;

pub use board::*;
pub use piece::*;
pub use search::*;
use std::io::{self, Write};

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
	let mut board = make_board();

	loop {
		println!();
		#[cfg(not(feature = "white"))]
		match get_check(&board) {
			None => {},
			Some(false) => println!("WHITE CHECKED\n"),
			Some(true) => println!("BLACK CHECKED\n"),
		}
		#[cfg(feature = "white")]
		match get_check(&board) {
			None => {},
			Some(true) => println!("WHITE CHECKED\n"),
			Some(false) => println!("BLACK CHECKED\n"),
		}
		println!("{}", board::board_string(&board));

		println!("Commands: move(f), spawn, clear, reset, score, possible, all, best");
		print!("> ");
		io::stdout().flush().unwrap();

		let mut cmd = String::new();
		match io::stdin().read_line(&mut cmd) {
			Ok(0) => break,
			Ok(ok) => ok,
			Err(err) => {
				eprintln!("Error reading line: {}", err);
				break;
			}
		};

		let mut args = cmd.split_whitespace();
		let cmd = match args.next() {
			Some(cmd) => cmd,
			None => continue,
		};
		let args: Vec<_> = args.collect();

		macro_rules! usage {
			($n:expr, $usage:expr) => {
				if args.len() != $n {
					eprintln!($usage);
					eprintln!("Incorrect arguments");
					continue;
				}
			}
		}
		macro_rules! parse_pos {
			($input:expr) => {
				match parse_position($input) {
					Some(pos) => pos,
					None => {
						eprintln!("Invalid position");
						continue;
					}
				}
			}
		}
		match cmd {
			"move" | "movef" => {
				usage!(2, "move(f) <from> <to>");

				let from = parse_pos!(args[0]);
				let to = parse_pos!(args[1]);

				if cmd == "move" {
					let piece = *board_get(&board, from);

					let mut found = false;
					for m in &piece.possible_moves(&board, from) {
						if *m == Some(to) {
							found = true;
						}
					}

					if !found {
						eprintln!("Can't move there!");
						eprintln!("TIP: movef moves without checking first.");
						continue;
					}
				}

				board_move(&mut board, from, to);
			},
			"spawn" => {
				usage!(2, "spawn <position> <piece>");

				let pos = parse_pos!(args[0]);
				let piece = match args[1].parse() {
					Ok(piece) => piece,
					Err(_) => {
						eprintln!("No such piece");
						continue;
					}
				};

				board_set(&mut board, pos, piece);
			},
			"clear" => {
				usage!(0, "clear");

				for line in &mut board {
					for piece in line.iter_mut() {
						*piece = Piece::Empty;
					}
				}
			},
			"reset" => {
				usage!(0, "reset");

				board = board::make_board();
			},
			"score" => {
				usage!(0, "score");

				println!("{}", score(&board));
			}
			"possible" => {
				usage!(1, "possible <pos>");

				let pos = parse_pos!(args[0]);

				let possible = (*board_get(&board, pos)).possible_moves(&board, pos);
				if possible.iter().all(|pos| pos.is_none()) {
					println!("No possible moves");
					continue;
				}

				println!("{}", positions_join(
					possible.iter()
						.filter(|pos| pos.is_some())
						.map(|pos| pos.unwrap())
				));
			},
			"all" => {
				usage!(0, "all");

				let possible = possible_moves(&board, true);
				if possible.is_empty() {
					println!("No possible moves");
					continue;
				}

				for ((x, y), moves) in possible {
					if moves.is_empty() {
						continue;
					}

					let mut pos = String::with_capacity(2);
					position_string((x, y), &mut pos);

					println!("{}: {}", pos, positions_join(
						moves.iter()
							.filter(|pos| pos.is_some())
							.map(|pos| pos.unwrap())
					));
				}
			},
			"best" => {
				usage!(0, "best");

				#[cfg(feature = "cpuprofiler")]
				PROFILER.lock().unwrap().start("crappy-chess-minimax.profile").unwrap();

				let (score, from, to) = search(&mut board, true, 0, std::i32::MIN, std::i32::MAX);

				#[cfg(feature = "cpuprofiler")]
				PROFILER.lock().unwrap().stop().unwrap();

				board_move(&mut board, from, to);

				println!("Final Score: {}", score);

				let mut string = String::with_capacity(2 + 4 + 2);
				position_string(from, &mut string);
				string.push_str(" to ");
				position_string(to, &mut string);
				println!("Move {}", string);
			},
			_ => eprintln!("Unknown command"),
		}
	}
}
