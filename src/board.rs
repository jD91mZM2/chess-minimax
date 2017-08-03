use *;
use std::collections::HashMap;

macro_rules! white {
	($name:ident) => {
		Piece::$name(false)
	}
}
macro_rules! black {
	($name:ident) => {
		Piece::$name(true)
	}
}
macro_rules! none {
	() => {
		Piece::Empty
	}
}

pub type Board = [[Piece; 8]; 8];

pub fn board_string(board: &Board) -> String {
	let mut output = String::with_capacity(8 * 9 + (9 + 2 + 1)); // width * height + (height + index + newline)

	output.push_str("  HGFEDCBA\n");
	for (i, line) in board.iter().enumerate() {
		output.push_str(&(i + 1).to_string());
		output.push(' ');
		for piece in line {
			output.push(piece.to_char());
		}
		output.push('\n');
	}

	output
}

pub fn possible_moves(board: &Board, black: bool) -> HashMap<(i8, i8), Vec<(i8, i8)>> {
	let mut map = HashMap::new();

	for (y, line) in board.iter().enumerate() {
		for (x, piece) in line.iter().enumerate() {
			if piece.is_black() != black || piece.is_empty() {
				continue;
			}

			let pos = (x as i8, y as i8);
			let moves = map.entry(pos).or_insert_with(|| Vec::new());

			for m in piece.possible_moves(board, pos) {
				moves.push(m);
			}
		}
	}

	map
}

pub fn is_check(
			board: &Board,
			black: bool,
			possible: &HashMap<(i8, i8), Vec<(i8, i8)>>
		) -> bool {
	for moves in possible.values() {
		for pos in moves {
			if let Piece::King(black2) = board[pos.1 as usize][pos.0 as usize] {
				if black == black2 {
					return true;
				}
			}
		}
	}

	false
}
pub fn get_check(board: &Board) -> Option<bool> {
	if is_check(board, false, &possible_moves(board, true)) {
		return Some(false);
	}
	if is_check(board, true, &possible_moves(board, false)) {
		return Some(true);
	}

	None
}

pub fn make_board() -> Board {
	[
		[ white!(Rook), white!(Knight), white!(Bishop), white!(King), white!(Queen), white!(Bishop), white!(Knight), white!(Rook) ],
		[ white!(Pawn), white!(Pawn),   white!(Pawn),   white!(Pawn), white!(Pawn),  white!(Pawn),   white!(Pawn),   white!(Pawn) ],
		[ none!(),      none!(),        none!(),        none!(),      none!(),       none!(),        none!(),        none!()      ],
		[ none!(),      none!(),        none!(),        none!(),      none!(),       none!(),        none!(),        none!()      ],
		[ none!(),      none!(),        none!(),        none!(),      none!(),       none!(),        none!(),        none!()      ],
		[ none!(),      none!(),        none!(),        none!(),      none!(),       none!(),        none!(),        none!()      ],
		[ black!(Pawn), black!(Pawn),   black!(Pawn),   black!(Pawn), black!(Pawn),  black!(Pawn),   black!(Pawn),   black!(Pawn) ],
		[ black!(Rook), black!(Knight), black!(Bishop), black!(King), black!(Queen), black!(Bishop), black!(Knight), black!(Rook) ],
	]
}
