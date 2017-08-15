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
pub type Pos  = (i8, i8);
pub type Diff = [Option<(Pos, Piece, Piece)>; 3];

pub fn board_string(board: &Board) -> String {
	let mut output = String::new();

	#[cfg(not(feature = "white"))]
	output.push_str("  H G F E D C B A\n");
	#[cfg(feature = "white")]
	output.push_str("  A B C D E F G H\n");
	for (i, line) in board.iter().enumerate() {
		#[cfg(not(feature = "white"))]
		let i = i + 1;
		#[cfg(feature = "white")]
		let i = 8 - i;

		output.push_str(&i.to_string());
		output.push(' ');
		for piece in line {
			output.push(piece.to_char());
			output.push(' ');
		}
		output.push('\n');
	}

	output
}
#[cfg(feature = "cache")]
pub fn board_bytes(board: &Board) -> [u8; 64] {
	let mut output = [0; 64];
	let mut index = 0;

	for line in board {
		for piece in line {
			output[index] = piece.to_byte();
			index += 1;
		}
	}

	output
}

pub fn en_passant_get_capture(board: &Board, mine: bool, pos: Pos) -> Option<Pos> {
	let (x, y) = pos;

	let capture;
	if mine  && y == 2 {
		capture = (x, y + 1);
	} else if !mine && y == 5 {
		capture = (x, y - 1);
	} else {
		return None;
	}
	return if board_get(board, capture) != Piece::Pawn(!mine) { None } else { Some(capture) };
}

pub enum PosUnwinder {
	Normal {
		old:  Piece,
		from: Pos,
		to:   Pos
	},
	Promoted {
		old_from: Piece,
		from:     Pos,
		old_to:   Piece,
		to:       Pos
	},
	EnPassant {
		old:  Piece,
		from: Pos,
		to:   Pos,
		passant_capture: Pos,
		passant_old:     Piece
	}
}
pub fn board_set(board: &mut Board, pos: Pos, piece: Piece) {
	board[pos.1 as usize][pos.0 as usize] = piece;
}
pub fn board_get(board: &Board, pos: Pos) -> Piece {
	board[pos.1 as usize][pos.0 as usize]
}
pub fn board_move(board: &mut Board, from: Pos, to: Pos) -> (Diff, bool) {
	let mut old_from = board_get(board, from);
	let     old_to   = board_get(board, to);

	let mut changed = [None; 3];
	let mut special = false;
	changed[0] = Some((from, old_from, Piece::Empty));
	changed[1] = Some((to, old_to, old_from));

	if let Piece::Pawn(mine) = old_from {
		if (mine && to.1 == 0) ||
			(!mine && to.1 == 7) {

			old_from = Piece::Queen(mine);
			special = true;
		} else if let Some(pos) = en_passant_get_capture(board, mine, to) {
			changed[2] = Some((pos, board_get(board, pos), Piece::Empty));
			board_set(board, pos, Piece::Empty);
			special = true;
		}
	}

	board_set(board, to, old_from);
	board_set(board, from, Piece::Empty);

	(changed, special)
}
pub fn board_apply(board: &mut Board, diff: Diff) {
	for entry in &diff {
		if let Some((pos, from, _)) = *entry {
			board_set(board, pos, from);
		}
	}
}

pub fn possible_moves(board: &Board, mine: bool) -> HashMap<Pos, Vec<Pos>> {
	let mut map = HashMap::new();

	for (y, line) in board.iter().enumerate() {
		for (x, piece) in line.iter().enumerate() {
			if piece.is_mine() != mine || piece.is_empty() {
				continue;
			}

			let pos = (x as i8, y as i8);
			let moves = piece.possible_moves(board, pos);
			if !moves.is_empty() {
				map.insert(pos, moves);
			}
		}
	}

	map
}

pub fn get_check(
			board: &Board,
			mine: bool,
			possible: &HashMap<Pos, Vec<Pos>>
		) -> Option<Pos> {
	for (from, moves) in possible {
		for pos in moves {
			if let Piece::King(mine2) = board_get(board, *pos) {
				if mine == mine2 {
					return Some(*from);
				}
			}
		}
	}

	None
}
pub enum CheckStatus {
	CheckMine(bool),
	CheckYour(bool),
	None,
}
pub fn check_status(board: &mut Board) -> CheckStatus {
	let possible_mine = possible_moves(board, true);
	let possible_your = possible_moves(board, false);

	for mine in &[true, false] {
		let mine = *mine;
		let mut mate = true;

		let possible = if mine { &possible_mine } else { &possible_your };

		for (from, moves) in possible {
			for to in moves {
				let (diff, _) = board_move(board, *from, *to);

				let possible = possible_moves(board, !mine);
				if get_check(board, mine, &possible).is_none() {
					mate = false;
				}

				board_apply(board, diff);
			}
		}

		if mate || get_check(board, mine, if mine { &possible_your } else { &possible_mine }).is_some() {
			if mine {
				return CheckStatus::CheckMine(mate);
			} else {
				return CheckStatus::CheckYour(mate);
			}
		}
	}

	CheckStatus::None
}

pub fn make_board() -> Board {
	[
		#[cfg(not(feature = "white"))]
		[ your!(Rook), your!(Knight), your!(Bishop), your!(King), your!(Queen), your!(Bishop), your!(Knight), your!(Rook) ],
		#[cfg(feature = "white")]
		[ your!(Rook), your!(Knight), your!(Bishop), your!(Queen), your!(King), your!(Bishop), your!(Knight), your!(Rook) ],

		[ your!(Pawn), your!(Pawn), your!(Pawn), your!(Pawn), your!(Pawn), your!(Pawn), your!(Pawn), your!(Pawn) ],
		[ none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!()     ],
		[ none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!()     ],
		[ none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!()     ],
		[ none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!(),     none!()     ],
		[ mine!(Pawn), mine!(Pawn), mine!(Pawn), mine!(Pawn), mine!(Pawn), mine!(Pawn), mine!(Pawn), mine!(Pawn) ],

		#[cfg(not(feature = "white"))]
		[ mine!(Rook), mine!(Knight), mine!(Bishop), mine!(King), mine!(Queen), mine!(Bishop), mine!(Knight), mine!(Rook) ],
		#[cfg(feature = "white")]
		[ mine!(Rook), mine!(Knight), mine!(Bishop), mine!(Queen), mine!(King), mine!(Bishop), mine!(Knight), mine!(Rook) ],
	]
}
