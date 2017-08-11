use *;
use std::time::Duration;
use std::{self, thread};
use websocket::OwnedMessage;
use websocket::client::sync::Client;
use websocket::message::CloseData;
use websocket::result::WebSocketError;
use websocket::stream::sync::TcpStream;
use websocket::sync::Server;

macro_rules! print_err {
	($err:expr, $action:expr) => {
		eprintln!(concat!("An error occured while ", $action, "."));
		eprintln!("Details: {:?}", $err);
	}
}

const IP_PORT: (&str, u16) = ("localhost", 27455);
const VERSION: &str = "v1";
const ACCEPT: &str = "ACCEPT";
const REFUSE: &str = "REFUSE";

pub fn main() {
	let server = Server::bind(IP_PORT).unwrap();

	for request in server.filter_map(Result::ok) {
		thread::spawn(move || {
			let mut client = match request.use_protocol("chess").accept() {
				Ok(ok) => ok,
				Err(err) => {
					print_err!(err, "accepting connection");
					return;
				},
			};

			macro_rules! close {
				($code:expr, $msg:expr) => {
					let _ = client.send_message(&OwnedMessage::Close(
						Some(CloseData::new($code, $msg))
					));
				}
			}
			macro_rules! bad_request {
				() => {
					close!(1003, "Bad request".to_string());
				}
			}
			macro_rules! attempt {
				($result:expr, $action:expr) => {
					match $result {
						Ok(ok) => ok,
						Err(err) => {
							print_err!(err, $action);
							close!(1011, "Sorry - my bad. But something went wrong.!".to_string());
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
					bad_request!();
					return;
				}
				if init.next() != Some(VERSION) {
					send!(REFUSE.to_string());
					continue;
				}
				send!(ACCEPT.to_string());
				break;
			}
			attempt!(client.stream_ref().set_read_timeout(Some(Duration::from_secs(2*60))), "setting timeout");

			let mut board = make_board();
			#[cfg(feature = "white")]
			{
				let (_, from, to) = search(&mut board, true, 0, std::i32::MIN, std::i32::MAX);
				board_move(&mut board, from, to);

				send!(format!("WHITE MOVE {} {}", position_string(from), position_string(to)));
			}
			#[cfg(not(feature = "white"))]
			send!("BLACK".to_string());

			let mut castling = 0;
			loop {
				let message = receive!();
				let mut args = message.split_whitespace();
				let cmd = match args.next() {
					Some(cmd) => cmd,
					None => {
						bad_request!();
						return;
					}
				};
				let args = args.collect::<Vec<&str>>();

				macro_rules! check_len {
					($expected:expr) => {
						if args.len() != $expected {
							bad_request!();
							return;
						}
					}
				}
				macro_rules! parse_pos {
					($str:expr) => {
						match parse_position($str) {
							Some(result) => result,
							None => {
								bad_request!();
								return;
							},
						}
					}
				}

				if cmd != "MOVE" && castling != 0 {
					bad_request!();
					return;
				}
				match cmd {
					"SWAP" => {
						check_len!(0);
						send!(REFUSE.to_string())
					},
					"MOVE" => {
						check_len!(2);
						let from = parse_pos!(args[0]);
						let to   = parse_pos!(args[1]);

						match do_move(from, to, &mut board, castling != 0) {
							MoveResult::Accept(changed) => {
								if changed {
									send!(format!("INTO-QUEEN {}", position_string(to)));
								}
								if castling == 0 {
									send!(ACCEPT.to_string());

									let (_, from, to) = search(&mut board, true, 0, std::i32::MIN, std::i32::MAX);
									let (_, _, changed) = board_move(&mut board, from, to);

									send!(format!("MOVE {} {}", position_string(from), position_string(to)));

									if changed {
										send!(format!("INTO-QUEEN {}", position_string(to)));
									}
								}
							},
							MoveResult::Check(pos) => {
								send!(REFUSE.to_string());
								send!(format!("HIGHLIGHT {}", position_string(pos)));
							},
							MoveResult::Refuse => send!(REFUSE.to_string()),
						}
						if castling > 0 {
							castling -= 1;
						}
					},
					"CASTLING" => {
						check_len!(0);
						castling = 2;
					},
					_ => {
						bad_request!();
						return;
					}
				}
			}
		});
	}
}

fn receive(client: &mut Client<TcpStream>) -> Result<Option<String>, Box<std::error::Error>> {
	match client.recv_message() {
		Ok(OwnedMessage::Close(_)) => {
			client.send_message(&OwnedMessage::Close(None))?;
			Ok(None)
		},
		Ok(OwnedMessage::Text(text)) => Ok(Some(text)),
		Ok(_) => Ok(None),
		Err(WebSocketError::NoDataAvailable) => Ok(None),
		Err(err) => Err(Box::new(err)),
	}
}

enum MoveResult {
	Accept(bool),
	Check((i8, i8)),
	Refuse
}
fn do_move(from: (i8, i8), to: (i8, i8), board: &mut Board, force: bool) -> MoveResult {
	if !force {
		let piece = *board_get(&board, from);
		if piece.is_mine() {
			return MoveResult::Refuse;
		}

		let mut found = false;
		for m in &piece.possible_moves(&board, from) {
			if *m == to {
				found = true;
			}
		}

		if !found {
			return MoveResult::Refuse;
		}
	}

	let (old_from, old_to, changed) = board_move(board, from, to);

	if !force {
		let possible = possible_moves(&board, true);
		if let Some(piece) = get_check(&board, false, &possible) {
			board_set(board, from, old_from);
			board_set(board, to, old_to);

			return MoveResult::Check(piece);
		}
	}
	MoveResult::Accept(changed)
}
