// use *;
use std::{self, thread};
use std::time::Duration;
use websocket::OwnedMessage;
use websocket::sync::Server;
use websocket::message::CloseData;
use websocket::client::sync::Client;
use websocket::stream::sync::TcpStream;

macro_rules! print_err {
	($err:expr, $action:expr) => {
		eprintln!(concat!("An error occured while ", $action, "."));
		eprintln!("Details: {:?}", $err);
	}
}

const ACCEPT: &str = "ACCEPT";
const REFUSE: &str = "REFUSE";

pub fn main() {
	let server = Server::bind("localhost:1234").unwrap();

	for request in server.filter_map(Result::ok) {
		thread::spawn(move || {
			let mut client = match request.use_protocol("chess").accept() {
				Ok(ok) => ok,
				Err(err) => {
					print_err!(err, "accepting connection");
					return;
				},
			};
			println!("Connection!");

			macro_rules! attempt {
				($result:expr, $action:expr) => {
					match $result {
						Ok(ok) => ok,
						Err(err) => {
							let _ = client.send_message(&OwnedMessage::Close(Some(CloseData::new(500,
								"Sorry - my bad. But something went wrong.!".to_string()
							))));
							print_err!(err, $action);
							return;
						},
					}
				}
			}
			macro_rules! receive {
				() => {
					match receive(&mut client) {
						Ok(Some(message)) => message,
						Ok(None) => return,
						Err(err) => {
							print_err!(err, "receiving message");
							return;
						}
					}
				}
			}
			macro_rules! send {
				($msg:expr) => {
					attempt!(client.send_message(&OwnedMessage::Text($msg)), "sending message");
				}
			}

			attempt!(client.stream_ref().set_read_timeout(Some(Duration::from_secs(5))), "setting timeout");
			loop {
				let init = receive!();
				let mut init = init.split_whitespace();
				if init.next() != Some("INIT") {
					return;
				}
				if init.next() != Some("v0") {
					send!(REFUSE.to_string());
					continue;
				}
				send!(ACCEPT.to_string());
				break;
			}
			attempt!(client.stream_ref().set_read_timeout(Some(Duration::from_secs(2*60))), "setting timeout");

			// let mut board = make_board();
		});
	}
}

fn receive(client: &mut Client<TcpStream>) -> Result<Option<String>, Box<std::error::Error>> {
	match client.recv_message()? {
		OwnedMessage::Close(_) => {
			client.send_message(&OwnedMessage::Close(None))?;
			return Ok(None);
		},
		OwnedMessage::Text(text) => Ok(Some(text)),
		_ => Ok(None),
	}
}
/*
#[cfg(not(feature = "white"))]
match check_status(&board) {
	None => {},
	Some(false) => println!("WHITE CHECKED\n"),
	Some(true) => println!("BLACK CHECKED\n"),
}
#[cfg(feature = "white")]
match check_status(&board) {
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

		let force = cmd == "movef";

		let from = parse_pos!(args[0]);
		let to = parse_pos!(args[1]);

		if !force {
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

		let (old_from, old_to) = board_move(&mut board, from, to);

		if !force {
			let possible = possible_moves(&board, true);
			if get_check(&board, false, &possible).is_some() {
				eprintln!("Can't move there! You'd place yourself in check!");
				eprintln!("TIP: movef moves without checking first.");

				board_set(&mut board, from, old_from);
				board_set(&mut board, to, old_to);
				continue;
			}
		}
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
*/
