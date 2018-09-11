use chess_minimax::{
    board::Board,
    Side
};
use std::io;

fn draw_board<W: io::Write>(out: &mut W, board: &Board) -> io::Result<()> {
    use termion::{color::{self, *}, style::{self, *}};
    write!(out, "  {}{}", Fg(Red), Bold)?;
    let mut white;
    let iter: Box<Iterator<Item = _>> = match board.side() {
        Side::White => {
            writeln!(out, " A  B  C  D  E  F  G  H{}", Fg(color::Reset))?;
            Box::new(board.iter().enumerate())
        },
        Side::Black => {
            writeln!(out, " H  G  F  E  D  C  B  A{}", Fg(color::Reset))?;
            Box::new(board.iter().enumerate().rev())
        }
    };
    for (i, row) in iter {
        white = i % 2 == 1;
        write!(out, "{}{}{} ", Fg(Red), 8-i, Fg(color::Reset))?;
        let iter: Box<Iterator<Item = _>> = match board.side() {
            Side::White => Box::new(row.into_iter()),
            Side::Black => Box::new(row.into_iter().rev()),
        };
        for piece in iter {
            let piece = piece.map(|p| p.to_char()).unwrap_or(' ');
            if white {
                write!(out, "{}{} {} ", Bg(White), Fg(Black), piece)?;
            } else {
                write!(out, "{}{} {} ", Bg(Black), Fg(White), piece)?;
            }
            white = !white;
        }
        writeln!(out, "{}{}", Bg(color::Reset), Fg(color::Reset))?;
    }
    write!(out, "{}", style::Reset);
    Ok(())
}

fn main() {
    let mut board = Board::new(Side::White);
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    draw_board(&mut stdout, &board).unwrap();

    board.each_piece(board.side(), |board, from| {
        print!("{}: ", from);
        let mut first = true;
        board.each_move_for(from, |_board, to| {
            if !first {
                print!(", ");
            }
            first = false;
            print!("{}", to);
            None::<()>
        });
        println!();
        None::<()>
    });
}
