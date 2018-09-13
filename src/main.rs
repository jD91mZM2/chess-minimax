use chess_minimax::{
    board::{Board, Change},
    serialize,
    Pos,
    Side
};
use failure::Error;
use rustyline::{error::ReadlineError, Editor};
use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{self, Write},
};
#[cfg(feature = "threads")]
use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread
};

const BOARD_FILE: &'static str = "saved_board";
const DEPTH: u8 = 6;

struct Session<W: Write> {
    out: W,
    board: Board,
    side: Side,
    undo: Option<Change>,
    highlight: HashSet<Pos>
}
impl<W: Write> Session<W> {
    fn draw(&mut self) -> io::Result<()> {
        use termion::{color::{self, *}, style::{self, *}};
        write!(self.out, "  {}{}", Fg(Red), Bold)?;
        let iter: Box<Iterator<Item = _>> = match self.side {
            Side::White => {
                writeln!(self.out, " A  B  C  D  E  F  G  H{}", Fg(color::Reset))?;
                Box::new(self.board.iter().enumerate())
            },
            Side::Black => {
                writeln!(self.out, " H  G  F  E  D  C  B  A{}", Fg(color::Reset))?;
                Box::new(self.board.iter().enumerate().rev())
            }
        };
        for (y, row) in iter {
            write!(self.out, "{}{}{} ", Fg(Red), 8-y, Fg(color::Reset))?;
            let iter: Box<Iterator<Item = _>> = match self.side {
                Side::White => Box::new(row.into_iter().enumerate()),
                Side::Black => Box::new(row.into_iter().enumerate().rev()),
            };
            for (x, piece) in iter {
                let white = (y % 2 == 0) == (x % 2 == 0);
                let piece = piece.map(|p| p.to_char()).unwrap_or(' ');
                if self.highlight.contains(&Pos(x as i8, y as i8)) {
                    write!(self.out, "{}{} {} ", Bg(Yellow), Fg(Black), piece)?;
                } else if white {
                    write!(self.out, "{}{} {} ", Bg(White), Fg(Black), piece)?;
                } else {
                    write!(self.out, "{}{} {} ", Bg(Black), Fg(White), piece)?;
                }
            }
            writeln!(self.out, "{}{}", Bg(color::Reset), Fg(color::Reset))?;
        }
        write!(self.out, "{}", style::Reset)?;
        Ok(())
    }
    fn possible(&mut self, from: Pos) -> io::Result<()> {
        write!(self.out, "{}: ", from)?;
        let mut first = true;
        let mut moves = self.board.moves_for(from);
        while let Some(to) = moves.next(&self.board) {
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
    fn command(&mut self, line: &str) -> io::Result<()> {
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
                while let Some(pos) = pieces.next(&self.board) {
                    self.possible(pos)?;
                }
            },
            Some("move") | Some("movef") => {
                expect!(args.len() == 2, "move(f) <from> <to>");

                let from = expect!(args[0].parse());
                let to = expect!(args[1].parse());

                if cmd == Some("move") {
                    let mut possible = false;
                    let mut moves = self.board.moves_for(from);
                    while let Some(m) = moves.next(&self.board) {
                        if m == to {
                            possible = true;
                        }
                    }
                    expect!(possible, "piece can't move there (hint: movef)");
                }

                self.undo = Some(self.board.move_(from, to));
            },
            Some("undo") => {
                expect!(args.is_empty(), "undo");

                match self.undo.take() {
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

                #[cfg(not(feature = "threads"))]
                let res = {
                    self.board.minimax(DEPTH, !self.side, None)
                };
                #[cfg(feature = "threads")]
                let res = {
                    let exit = Arc::new(AtomicBool::new(false));

                    println!("Calculating, press ENTER to stop:");

                    let thread = {
                        let side = self.side;
                        let mut board = self.board.clone();
                        let exit = Arc::clone(&exit);
                        thread::spawn(move || -> Result<_, io::Error> {
                            let mut res = None;
                            for i in DEPTH - 3.. {
                                writeln!(io::stdout(), "Trying depth {}...", i)?;
                                if let Some(new) = board.minimax(i, !side, Some(&exit)) {
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
                    self.board.move_(res.from, res.to);
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

fn main() -> Result<(), Error> {
    let board = Board::new();
    let stdout = io::stdout();
    #[cfg(not(feature = "threads"))]
    let stdout = stdout.lock();

    let mut s = Session {
        out: stdout,
        board,
        side: Side::Black,
        undo: None,
        highlight: HashSet::new()
    };

    let mut editor = Editor::<()>::new();

    loop {
        s.draw()?;

        let line = match editor.readline("> ") {
            Err(ReadlineError::Interrupted)
            | Err(ReadlineError::Eof) => break,
            result => result?
        };

        s.highlight.clear();

        if let Err(err) = s.command(&line) {
            writeln!(s.out, "error: {}", err)?;
        }
        editor.add_history_entry(line);
    }
    Ok(())
}
