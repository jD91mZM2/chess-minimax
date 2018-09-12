use chess_minimax::{
    board::{Board, Change},
    Pos,
    Side
};
use failure::Error;
use rustyline::{error::ReadlineError, Editor};
use std::io::{self, Write};

struct Session<W: Write> {
    out: W,
    board: Board,
    undo: Option<Change>
}
impl<W: Write> Session<W> {
    fn draw(&mut self) -> io::Result<()> {
        use termion::{color::{self, *}, style::{self, *}};
        write!(self.out, "  {}{}", Fg(Red), Bold)?;
        let mut white;
        let iter: Box<Iterator<Item = _>> = match self.board.side() {
            Side::White => {
                writeln!(self.out, " A  B  C  D  E  F  G  H{}", Fg(color::Reset))?;
                Box::new(self.board.iter().enumerate())
            },
            Side::Black => {
                writeln!(self.out, " H  G  F  E  D  C  B  A{}", Fg(color::Reset))?;
                Box::new(self.board.iter().enumerate().rev())
            }
        };
        for (i, row) in iter {
            white = i % 2 == 0;
            write!(self.out, "{}{}{} ", Fg(Red), 8-i, Fg(color::Reset))?;
            let iter: Box<Iterator<Item = _>> = match self.board.side() {
                Side::White => Box::new(row.into_iter()),
                Side::Black => Box::new(row.into_iter().rev()),
            };
            for piece in iter {
                let piece = piece.map(|p| p.to_char()).unwrap_or(' ');
                if white {
                    write!(self.out, "{}{} {} ", Bg(White), Fg(Black), piece)?;
                } else {
                    write!(self.out, "{}{} {} ", Bg(Black), Fg(White), piece)?;
                }
                white = !white;
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
            ($($arg:tt),*) => {
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

                let mut pieces = self.board.pieces(self.board.side());
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
            }
            Some(_) => println!("unknown command"),
        }
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let board = Board::new(Side::White);
    let stdout = io::stdout();
    let stdout = stdout.lock();

    let mut s = Session {
        out: stdout,
        board,
        undo: None
    };

    let mut editor = Editor::<()>::new();

    loop {
        s.draw()?;

        let line = match editor.readline("> ") {
            Err(ReadlineError::Interrupted)
            | Err(ReadlineError::Eof) => break,
            result => result?
        };

        s.command(&line)?;
    }
    Ok(())
}
