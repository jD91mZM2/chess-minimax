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

impl Direction {
	pub fn all() -> [Direction; 8] {
		[
			Direction::UpLeft,
			Direction::UpRight,
			Direction::LeftUp,
			Direction::LeftDown,
			Direction::RightUp,
			Direction::RightDown,
			Direction::DownLeft,
			Direction::DownRight
		]
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

fn position_string(input: (i8, i8), output: &mut String) {
	output.push(std::char::from_u32((7 - input.0) as u32 + 'A' as u32).unwrap());
	output.push(std::char::from_u32(input.1 as u32 + '1' as u32).unwrap());
}
fn positions_join<'a, I: Iterator<Item = &'a (i8, i8)>>(input: I, len: usize) -> String {
	let mut output = String::with_capacity(len * (2 + 2) - 2);
	let mut first  = true;

	for pos in input {
		if first {
			first = false
		} else {
			output.push_str(", ");
		}

		position_string(*pos, &mut output)
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
	let mut board = board::make_board();
	let mut auto_player = None;

	loop {
		println!();
		println!("{}", board::board_string(&board));

		println!();
		match get_check(&board) {
			None => {},
			Some(false) => println!("WHITE CHECKED"),
			Some(true) => println!("BLACK CHECKED"),
		}

		println!("Commands: move, possible, all, best, reset");
		print!("> ");
		io::stdout().flush().unwrap();

		if let Some(ref mut auto_player) = auto_player {
			let (_, from, to) = search(&mut board, *auto_player, 0);
			*auto_player = !*auto_player;

			board[to.1 as usize][to.0 as usize] = board[from.1 as usize][from.0 as usize];
			board[from.1 as usize][from.0 as usize] = Piece::Empty;
			continue;
		}

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
			"move" => {
				usage!(2, "move <from> <to>");

				let from = parse_pos!(args[0]);
				let to = parse_pos!(args[1]);

				let (from_x, from_y) = from;
				let (to_x, to_y) = to;

				board[to_y as usize][to_x as usize] = board[from_y as usize][from_x as usize];
				board[from_y as usize][from_x as usize] = Piece::Empty;
			},
			"possible" => {
				usage!(1, "possible <pos>");

				let pos = parse_pos!(args[0]);
				let (x, y) = pos;

				let possible = board[y as usize][x as usize].possible_moves(&board, pos);
				if possible.is_empty() {
					println!("No possible moves");
					continue;
				}

				println!("{}", positions_join(possible.iter(), possible.len()));
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

					println!("{}: {}", pos, positions_join(moves.iter(), moves.len()));
				}
			},
			"best" => {
				usage!(0, "best");

				let (score, from, to) = search(&mut board, true, 0);

				board[to.1 as usize][to.0 as usize] = board[from.1 as usize][from.0 as usize];
				board[from.1 as usize][from.0 as usize] = Piece::Empty;

				println!("Final Score: {}", score);

				let mut string = String::with_capacity(2 + 4 + 2);
				position_string(from, &mut string);
				string.push_str(" to ");
				position_string(to, &mut string);
				println!("Move {}", string);
			},
			"reset" => {
				usage!(0, "reset");

				board = board::make_board();
			},
			"play" => {
				auto_player = Some(true);
			}
			_ => eprintln!("Unknown command"),
		}
	}
}
