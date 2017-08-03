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

fn string_position(input: (i8, i8), output: &mut String) {
	println!("{:?}", input);
	output.push(std::char::from_u32((7 - input.0) as u32 + 'A' as u32).unwrap());
	output.push(std::char::from_u32(input.1 as u32 + '1' as u32).unwrap());
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

	println!("{}, {}", x, y);
	Some((x, y))
}

fn main() {
	let mut board = board::make_board();

	loop {
		println!();
		println!("{}", board::board_string(&board));

		println!("Commands: move, possible, all, check, best, reset");
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
		match &*cmd {
			"move" => {
				usage!(2, "move <from> <to>");

				let from = parse_pos!(&args[0]);
				let to = parse_pos!(&args[1]);

				let (from_x, from_y) = from;
				let (to_x, to_y) = to;

				board[to_y as usize][to_x as usize] = board[from_y as usize][from_x as usize];
				board[from_y as usize][from_x as usize] = Piece::Empty;
			},
			"possible" => {
				usage!(1, "possible <pos>");

				let pos = parse_pos!(&args[0]);
				let (x, y) = pos;

				let possible = board[y as usize][x as usize].possible_moves(&board, pos);
				if possible.is_empty() {
					println!("No possible moves");
					continue;
				}

				let mut output = String::with_capacity(possible.len() * 4 - 2);
				let mut first  = true;

				for pos in possible {
					if first {
						first = false
					} else {
						output.push_str(", ");
					}

					string_position(pos, &mut output)
				}

				println!("{}", output);
			},
			"all" => {
				usage!(0, "all");

				let possible = possible_moves(&board, true);
				if possible.is_empty() {
					println!("No possible moves");
					continue;
				}

				let mut output = String::with_capacity(possible.len() * 4 - 2);
				let mut first  = true;

				for moves in possible.values() {
					for pos in moves {
						if first {
							first = false
						} else {
							output.push_str(", ");
						}

						string_position(*pos, &mut output)
					}
				}

				println!("{}", output);
			},
			"check" => {
				usage!(0, "check");

				match get_check(&board) {
					None => println!("No check"),
					Some(false) => println!("WHITE CHECKED"),
					Some(true) => println!("BLACK CHECKED"),
				}
			},
			"best" => {
				usage!(0, "best");

				let (score, from, to) = search(&mut board, true, 0);

				board[to.1 as usize][to.0 as usize] = board[from.1 as usize][from.0 as usize];
				board[from.1 as usize][from.0 as usize] = Piece::Empty;

				println!("Final Score: {}", score);

				let mut string = String::with_capacity(2 + 4 + 2);
				string_position(from, &mut string);
				string.push_str(" to ");
				string_position(to, &mut string);
				println!("Move {}", string);
			},
			"reset" => {
				usage!(0, "reset");

				board = board::make_board();
			}
			_ => eprintln!("Unknown command"),
		}
	}
}
