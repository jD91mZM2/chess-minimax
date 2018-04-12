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

#[cfg(not(feature = "public"))]
const IP_PORT: (&str, u16) = ("localhost", 27455);
#[cfg(feature = "public")]
const IP_PORT: (&str, u16) = ("0.0.0.0", 27455);

const VERSION: &str = "v2";
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
                let (_, from, to) = start_search(&mut board);
                board_move(&mut board, from, to);

                send!(format!("WHITE MOVE {} {}", position_string(from), position_string(to)));
            }
            #[cfg(not(feature = "white"))]
            send!("BLACK".to_string());

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

                if cmd != "MOVE" {
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

                        match do_move(from, to, &mut board) {
                            MoveResult::Accept((diff, special)) => {
                                if let Some(string) = checkmate_status_string(&mut board, true) {
                                    send!(string.to_string());
                                    close!(1000, String::new());
                                }
                                send!(ACCEPT.to_string());
                                if special {
                                    send!(format!("DIFF {}", diff_string(diff)));
                                }

                                let (_, from, to) = start_search(&mut board);
                                let (diff, special) = board_move(&mut board, from, to);

                                if special {
                                    send!(format!("DIFF {}", diff_string(diff)));
                                } else {
                                    send!(format!("MOVE {} {}", position_string(from), position_string(to)));
                                }

                                if let Some(string) = checkmate_status_string(&mut board, false) {
                                    send!(string.to_string());
                                    close!(1000, String::new());
                                }
                            },
                            MoveResult::Check(pos) => {
                                send!(REFUSE.to_string());
                                send!(format!("HIGHLIGHT {}", position_string(pos)));
                            },
                            MoveResult::Refuse => send!(REFUSE.to_string()),
                        }
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
fn checkmate_status_string(board: &mut Board, my_turn: bool) -> Option<&'static str> {
    let status = check_status(board);
    if !my_turn {
        if let CheckStatus::CheckMine(false) = status {
            // It's still in check after its move.
            // Knows it's going to lose either way.
            return Some("I-GIVE-UP")
        }
    }

    #[cfg(not(feature = "white"))]
    match status {
        CheckStatus::CheckMine(true) => Some("CHECKMATE BLACK"),
        CheckStatus::CheckYour(true) => Some("CHECKMATE WHITE"),
        _ => None,
    }
    #[cfg(feature = "white")]
    match check_status(board) {
        CheckStatus::CheckMine(true) => Some("CHECKMATE WHITE"),
        CheckStatus::CheckYour(true) => Some("CHECKMATE BLACK"),
        _ => None,
    }
}

enum MoveResult {
    Accept((Diff, bool)),
    Check(Pos),
    Refuse
}
fn do_move(from: Pos, to: Pos, board: &mut Board) -> MoveResult {
    // if !force {
    let piece = board_get(&board, from);
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
    // }

    let (diff, special) = board_move(board, from, to);

    // if !force {
    let possible = possible_moves(&board, true);
    if let Some(piece) = get_check(&board, false, &possible) {
        board_apply(board, diff);

        return MoveResult::Check(piece);
    }
    // }
    MoveResult::Accept((diff, special))
}
fn diff_string(diff: Diff) -> String {
    let mut output = String::new();

    for entry in &diff {
        if let Some((pos, _, to)) = *entry {
            if !output.is_empty() {
                output.push(' ');
            }
            output.push_str(&position_string(pos));
            output.push(' ');
            output.push_str(to.to_str());
        }
    }

    output
}
