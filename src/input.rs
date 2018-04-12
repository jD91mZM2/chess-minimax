use *;
use std::io::{self, Write};

fn positions_join<'a, I: Iterator<Item = &'a Pos>>(input: I) -> String {
    let mut output = String::new();
    let mut first  = true;

    for pos in input {
        if first {
            first = false
        } else {
            output.push_str(", ");
        }

        output.push_str(&position_string(*pos));
    }

    output
}

pub fn main() {
    let mut board = make_board();

    let stdin = io::stdin(); stdin.lock();
    let mut stdout = io::stdout(); stdout.lock();
    let mut stderr = io::stderr(); stderr.lock();

    let mut prompt = Prompt {
        board: &mut board,
        stdout: &mut stdout,
        stderr: &mut stderr
    };

    loop {
        prompt.print_prompt().unwrap();

        let mut cmd = String::new();
        match stdin.read_line(&mut cmd) {
            Ok(0) => break,
            Ok(ok) => ok,
            Err(err) => {
                writeln!(prompt.stderr, "Error reading line: {}", err).unwrap();
                break;
            }
        };

        prompt.apply_input(&cmd).unwrap();
    }
}

pub struct Prompt<'a, Out: Write + 'a, Err: Write + 'a> {
    board: &'a mut Board,
    stdout: &'a mut Out,
    stderr: &'a mut Err
}
impl<'a, Out: Write + 'a, Err: Write + 'a> Prompt<'a, Out, Err> {
    pub fn print_prompt(&mut self) -> io::Result<()> {
        let board = &mut *self.board;
        let out = &mut *self.stdout;

        #[cfg(not(feature = "white"))]
        match check_status(board) {
            CheckStatus::None => {},
            CheckStatus::CheckMine(mate) => writeln!(out, "BLACK CHECK{}\n", if mate { "MATE" } else { "ED" })?,
            CheckStatus::CheckYour(mate) => writeln!(out, "WHITE CHECK{}\n", if mate { "MATE" } else { "ED" })?,
        }
        #[cfg(feature = "white")]
        match check_status(board) {
            CheckStatus::None => {},
            CheckStatus::CheckMine(mate) => writeln!(out, "WHITE CHECK{}\n", if mate { "MATE" } else { "ED" })?,
            CheckStatus::CheckYour(mate) => writeln!(out, "BLACK CHECK{}\n", if mate { "MATE" } else { "ED" })?,
        }
        writeln!(out, "{}", board::board_string(board))?;

        writeln!(out, "Commands: move(f), spawn, clear, reset, score, possible, all, best")?;
        write!(out, "> ")?;
        out.flush()
    }
    pub fn apply_input(&mut self, cmd: &str) -> io::Result<()> {
        let board = &mut *self.board;
        let out = &mut *self.stdout;
        let err = &mut *self.stderr;

        let mut args = cmd.split_whitespace();
        let cmd = match args.next() {
            Some(cmd) => cmd,
            None => return Ok(()),
        };
        let args: Vec<_> = args.collect();

        macro_rules! usage {
            ($n:expr, $usage:expr) => {
                if args.len() != $n {
                    writeln!(err, $usage)?;
                    writeln!(err, "Incorrect arguments")?;
                    return Ok(());
                }
            }
        }
        macro_rules! parse_pos {
            ($input:expr) => {
                match parse_position($input) {
                    Some(pos) => pos,
                    None => {
                        writeln!(err, "Invalid position")?;
                        return Ok(());
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
                    let piece = board_get(board, from);

                    if piece.is_mine() {
                        writeln!(err, "Can't move with that piece! It's mine!")?;
                        writeln!(err, "TIP: movef moves without checking first.")?;
                        return Ok(());
                    }

                    let mut found = false;
                    for m in &piece.possible_moves(board, from) {
                        if *m == to {
                            found = true;
                        }
                    }

                    if !found {
                        writeln!(err, "Can't move there!")?;
                        writeln!(err, "TIP: movef moves without checking first.")?;
                        return Ok(());
                    }
                }

                let (diff, _) = board_move(board, from, to);

                if !force {
                    let possible = possible_moves(board, true);
                    if get_check(board, false, &possible).is_some() {
                        writeln!(err, "Can't move there! You'd place yourself in check!")?;
                        writeln!(err, "TIP: movef moves without checking first.")?;

                        board_apply(board, diff);
                        return Ok(());
                    }
                }
            },
            "spawn" => {
                usage!(2, "spawn <position> <piece>");

                let pos = parse_pos!(args[0]);
                let piece = match args[1].parse() {
                    Ok(piece) => piece,
                    Err(_) => {
                        writeln!(err, "No such piece")?;
                        return Ok(());
                    }
                };

                board_set(board, pos, piece);
            },
            "clear" => {
                usage!(0, "clear");

                for line in board {
                    for piece in line.iter_mut() {
                        *piece = Piece::Empty;
                    }
                }
            },
            "reset" => {
                usage!(0, "reset");

                *board = board::make_board();
            },
            "score" => {
                usage!(0, "score");

                writeln!(out, "{}", score(board))?;
            }
            "possible" => {
                usage!(1, "possible <pos>");

                let pos = parse_pos!(args[0]);

                let possible = board_get(board, pos).possible_moves(board, pos);
                if possible.is_empty() {
                    writeln!(out, "No possible moves")?;
                    return Ok(());
                }

                writeln!(out, "{}", positions_join(possible.iter()))?;
            },
            "all" => {
                usage!(0, "all");

                let possible = possible_moves(board, true);
                if possible.is_empty() {
                    writeln!(out, "No possible moves")?;
                    return Ok(());
                }

                for ((x, y), moves) in possible {
                    if moves.is_empty() {
                        return Ok(());
                    }

                    let pos = position_string((x, y));
                    writeln!(out, "{}: {}", pos, positions_join(moves.iter()))?;
                }
            },
            "best" => {
                usage!(0, "best");

                #[cfg(feature = "cpuprofiler")]
                PROFILER.lock().unwrap().start("crappy-chess-minimax.profile").unwrap();

                let (score, from, to) = start_search(board);

                #[cfg(feature = "cpuprofiler")]
                PROFILER.lock().unwrap().stop().unwrap();

                board_move(board, from, to);

                writeln!(out, "Final Score: {}", score)?;
                writeln!(out, "Move {} to {}", position_string(from), position_string(to))?;
            },
            _ => writeln!(err, "Unknown command")?,
        }
        Ok(())
    }
}
