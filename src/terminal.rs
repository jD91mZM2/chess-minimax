use crate::{
    board::{Board, Change},
    piece::PieceKind,
    serialize,
    Pos,
    Side
};
use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{self, Write},
};
#[cfg(feature = "terminal-bin")]
use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread
};

const BOARD_FILE: &'static str = "saved_board";
const DEPTH: u8 = 5;

// Not using termion because I REALLY want to be able to use this front-end in
// WASM.
const BOLD: &str = "\x1b[1m";
const ITALIC: &str = "\x1b[3m";
const RESET: &str = "\x1b[0m";

const BLACK_BG: &str = "\x1b[40m";
const BLACK_FG: &str = "\x1b[30m";
const RED_FG: &str = "\x1b[31m";
const WHITE_BG: &str = "\x1b[47m";
const WHITE_FG: &str = "\x1b[37m";
const YELLOW_BG: &str = "\x1b[43m";

pub struct Session<W: Write> {
    pub out: W,
    pub board: Board,
    pub side: Side,
    pub undo: Vec<Change>,
    pub highlight: HashSet<Pos>
}
impl<W: Write> Session<W> {
    pub fn check_status(&mut self, side: Side) -> io::Result<()> {
        let side_str = match side {
            Side::Black => "black",
            Side::White => "white"
        };
        if self.board.iter()
                .flat_map(|row| row.iter())
                .filter_map(|piece| piece.as_ref())
                .all(|p| p.kind != PieceKind::King || p.side != side) {
            writeln!(self.out, "{}{} has no king{}", ITALIC, side_str, RESET)?;
        } else if self.board.is_checkmate(side) {
            writeln!(self.out, "{}{} is checkmated{}", ITALIC, side_str, RESET)?;
        } else if let Some(pos) = self.board.check(side) {
            writeln!(self.out, "{}{} is checked by {}{}", ITALIC, side_str, pos, RESET)?;
        }
        Ok(())
    }
    pub fn draw(&mut self) -> io::Result<()> {
        write!(self.out, "  {}{}", RED_FG, BOLD)?;
        let iter: Box<Iterator<Item = _>> = match self.side {
            Side::White => {
                write!(self.out, " A  B  C  D  E  F  G  H")?;
                Box::new(self.board.iter().enumerate())
            },
            Side::Black => {
                write!(self.out, " H  G  F  E  D  C  B  A")?;
                Box::new(self.board.iter().enumerate().rev())
            }
        };
        writeln!(self.out, "{}", RESET)?;
        for (y, row) in iter {
            write!(self.out, "{}{}{}{}{} ", BOLD, RED_FG, 8-y, RESET, BOLD)?;
            let iter: Box<Iterator<Item = _>> = match self.side {
                Side::White => Box::new(row.into_iter().enumerate()),
                Side::Black => Box::new(row.into_iter().enumerate().rev()),
            };
            for (x, piece) in iter {
                let white = (y % 2 == 0) == (x % 2 == 0);
                let piece = piece.map(|p| p.to_char()).unwrap_or(' ');
                if self.highlight.contains(&Pos(x as i8, y as i8)) {
                    write!(self.out, "{}{} {} ", YELLOW_BG, BLACK_FG, piece)?;
                } else if white {
                    write!(self.out, "{}{} {} ", WHITE_BG, BLACK_FG, piece)?;
                } else {
                    write!(self.out, "{}{} {} ", BLACK_BG, WHITE_FG, piece)?;
                }
            }
            writeln!(self.out, "{}", RESET)?;
        }

        writeln!(self.out)?;

        self.check_status(Side::Black)?;
        self.check_status(Side::White)?;

        writeln!(self.out, "{}Possible commands: \
            all, \
            go, \
            load, \
            move(f), \
            possible, \
            rotate, \
            save, \
            score, \
            undo\
            {}", ITALIC, RESET)?;

        Ok(())
    }
    pub fn possible(&mut self, from: Pos) -> io::Result<()> {
        write!(self.out, "{}: ", from)?;
        let mut first = true;
        let mut moves = self.board.moves_for(from);
        while let Some(to) = moves.next(&mut self.board) {
            self.highlight.insert(to);
            if !first {
                write!(self.out, ", ")?;
            }
            first = false;
            write!(self.out, "{}", to)?;
        };
        writeln!(self.out)?;
        Ok(())
    }
    pub fn command(&mut self, line: &str) -> io::Result<()> {
        self.highlight.clear();

        macro_rules! println {
            ($($arg:expr),*) => {
                writeln!(self.out$(, $arg)*)?;
            }
        }
        macro_rules! expect {
            ($cond:expr, $err:expr) => {
                if !$cond {
                    println!($err);
                    return Ok(());
                }
            };
            ($val:expr) => {
                match $val {
                    Ok(pos) => pos,
                    Err(err) => {
                        println!("{}", err);
                        return Ok(());
                    }
                }
            };
        }

        let mut args = line.split_whitespace();
        let cmd = args.next();
        let args: Vec<_> = args.collect();

        match cmd.as_ref().map(|s| &**s) {
            None => (),
            Some("possible") => {
                expect!(args.len() == 1, "possible <position>");

                let pos = expect!(args[0].parse());
                self.possible(pos)?;
            },
            Some("all") => {
                expect!(args.is_empty(), "all");

                let mut pieces = self.board.pieces(self.side);
                while let Some((pos, _)) = pieces.next(&self.board) {
                    self.possible(pos)?;
                }
            },
            Some("move") | Some("movef") => {
                expect!(args.len() == 2, "move(f) <from> <to>");

                let force = cmd == Some("movef");

                let from = expect!(args[0].parse());
                let to = expect!(args[1].parse());

                if !force {
                    let mut possible = false;
                    let mut moves = self.board.moves_for(from);
                    while let Some(m) = moves.next(&mut self.board) {
                        if m == to {
                            possible = true;
                            break;
                        }
                    }
                    expect!(possible, "piece can't move there (hint: movef)");
                }

                let undo = self.board.move_(from, to);
                if !force {
                    if let Some(pos) = self.board.check(!self.side) {
                        self.board.undo(undo);
                        self.highlight.insert(pos);
                        println!("can't place yourself in check!");
                        return Ok(());
                    }
                }
                self.undo.push(undo);
            },
            Some("undo") => {
                expect!(args.is_empty(), "undo");

                match self.undo.pop() {
                    Some(change) => self.board.undo(change),
                    None => {
                        println!("no recent move to undo");
                    }
                }
            },
            Some("score") => {
                println!("Black score: {}", self.board.score(Side::Black));
                println!("White score: {}", self.board.score(Side::White));
            },
            Some("go") => {
                expect!(args.is_empty(), "go");

                #[cfg(not(feature = "terminal-bin"))]
                let res = {
                    self.board.minimax(DEPTH, self.side, None)
                };
                #[cfg(feature = "terminal-bin")]
                let res = {
                    let exit = Arc::new(AtomicBool::new(false));

                    println!("Calculating, press ENTER to stop:");

                    let thread = {
                        let side = self.side;
                        let mut board = self.board.clone();
                        let exit = Arc::clone(&exit);
                        thread::spawn(move || -> io::Result<_> {
                            let mut res = None;
                            for i in DEPTH - 3.. {
                                writeln!(io::stdout(), "Trying depth {}...", i)?;
                                if let Some(new) = board.minimax(i, side, Some(&exit)) {
                                    res = Some((i, new));
                                }
                                if exit.load(Ordering::SeqCst) {
                                    break;
                                }
                            }
                            Ok(res)
                        })
                    };

                    let mut buffer = String::new();
                    io::stdin().read_line(&mut buffer)?;

                    println!("Stopping...");
                    exit.store(true, Ordering::SeqCst);

                    match thread.join().unwrap()? {
                        Some((depth, res)) => {
                            println!("searched at depth {}", depth);
                            Some(res)
                        },
                        None => {
                            println!("nothing to do");
                            None
                        }
                    }
                };
                if let Some(res) = res {
                    let undo = self.board.move_(res.from, res.to);
                    self.undo.push(undo);
                    println!("move {} to {}", res.from, res.to);
                    println!("final score: {}", res.score);
                }
            },
            Some("rotate") => {
                expect!(args.is_empty(), "rotate");

                self.side = !self.side;
            },
            Some("save") => {
                expect!(args.is_empty(), "save");

                let mut file = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(BOARD_FILE)?;
                serialize::serialize_board(&mut file, &self.board)?;
            },
            Some("load") => {
                expect!(args.is_empty(), "load");

                let mut file = File::open(BOARD_FILE)?;
                self.board = serialize::deserialize_board(&mut file)?;
            },
            Some(_) => println!("unknown command"),
        }
        Ok(())
    }
}
