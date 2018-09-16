use chess_minimax::{
    board::Board,
    terminal::Session,
    Side
};
use std::{
    collections::HashSet,
    io::{self, Write}
};
use failure::Error;
use rustyline::{error::ReadlineError, Editor};

fn main() -> Result<(), Error> {
    let board = Board::new();
    let stdout = io::stdout();

    let mut s = Session {
        out: stdout,
        board,
        side: Side::Black,
        undo: Vec::new(),
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

        if let Err(err) = s.command(&line) {
            writeln!(s.out, "error: {}", err)?;
        }
        editor.add_history_entry(line);
    }
    Ok(())
}
