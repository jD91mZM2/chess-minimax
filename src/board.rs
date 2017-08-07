use *;
use std::collections::HashMap;

macro_rules! your {
	($name:ident) => {
		Piece::$name(false)
	}
}
macro_rules! mine {
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
	let mut output = String::new();

	#[cfg(not(feature = "white"))]
	output.push_str("  H G F E D C B A\n");
	#[cfg(feature = "white")]
	output.push_str("  A B C D E F G H\n");
	for (i, line) in board.iter().enumerate() {
		output.push_str(&(i + 1).to_string());
		output.push(' ');
		for piece in line {
			output.push(piece.to_char());
			output.push(' ');
		}
		output.push('\n');
	}

	output
}

pub fn board_set(board: &mut Board, pos: (i8, i8), mut piece: Piece) {
	if let Piece::Pawn(mine) = piece {
		if (mine && pos.1 == 0) ||
			(!mine && pos.1 == 7) {

			piece = Piece::Queen(mine);
		}
	}
	board[pos.1 as usize][pos.0 as usize] = piece;
}
pub fn board_get(board: &Board, pos: (i8, i8)) -> &Piece {
	&board[pos.1 as usize][pos.0 as usize]
}
pub fn board_move(board: &mut Board, from: (i8, i8), to: (i8, i8)) -> (Piece, Piece) {
	let piece = *board_get(board, from);
	let old = *board_get(board, to);
	board_set(board, to, piece);
	board_set(board, from, Piece::Empty);

	(piece, old)
}

pub fn possible_moves(board: &Board, mine: bool) -> HashMap<(i8, i8), Vec<(i8, i8)>> {
	let mut map = HashMap::new();

	for (y, line) in board.iter().enumerate() {
		for (x, piece) in line.iter().enumerate() {
			if piece.is_mine() != mine || piece.is_empty() {
				continue;
			}

			let pos = (x as i8, y as i8);
			let moves = map.entry(pos).or_insert_with(Vec::new);

			for m in piece.possible_moves(board, pos) {
				moves.push(m);
			}
		}
	}

	map
}

pub fn is_check(
			board: &Board,
			mine: bool,
			possible: &HashMap<(i8, i8), Vec<(i8, i8)>>
		) -> bool {
	for moves in possible.values() {
		for pos in moves {
			if let Piece::King(mine2) = *board_get(board, *pos) {
				if mine == mine2 {
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
		#[cfg(not(feature = "white"))]
		[ your!(Rook), your!(Knight), your!(Bishop), your!(Queen), your!(King), your!(Bishop), your!(Knight), your!(Rook) ],
		#[cfg(feature = "white")]
		[ your!(Rook), your!(Knight), your!(Bishop), your!(King), your!(Queen), your!(Bishop), your!(Knight), your!(Rook) ],

		[ your!(Pawn), your!(Pawn), your!(Pawn), your!(Pawn), your!(Pawn), your!(Pawn), your!(Pawn), your!(Pawn) ],
		[ none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!()     ],
		[ none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!()     ],
		[ none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!()     ],
		[ none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!()     ],
		[ mine!(Pawn), mine!(Pawn), mine!(Pawn), mine!(Pawn), mine!(Pawn), mine!(Pawn), mine!(Pawn), mine!(Pawn) ],

		#[cfg(not(feature = "white"))]
		[ mine!(Rook), mine!(Knight), mine!(Bishop), mine!(Queen), mine!(King), mine!(Bishop), mine!(Knight), mine!(Rook) ],
		#[cfg(feature = "white")]
		[ mine!(Rook), mine!(Knight), mine!(Bishop), mine!(King), mine!(Queen), mine!(Bishop), mine!(Knight), mine!(Rook) ],
	]
}
