use *;

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
	let mut output = String::with_capacity(8 * 8 + 8); // width * height + newlines

	for row in board {
		for col in row {
			output.push(col.to_char());
		}
		output.push('\n');
	}

	output
}

pub fn make_board() -> Board {
	[
		[ your!(Rook), your!(Knight), your!(Bishop), your!(King), your!(Queen), your!(Bishop), your!(Knight), your!(Rook) ],
		[ your!(Pawn), your!(Pawn),   your!(Pawn),   your!(Pawn), your!(Pawn),  your!(Pawn),   your!(Pawn),   your!(Pawn) ],
		[ none!(),     none!(),       none!(),       none!(),     none!(),      none!(),       none!(),       none!()     ],
		[ none!(),     none!(),       none!(),       none!(),     none!(),      none!(),       none!(),       none!()     ],
		[ none!(),     none!(),       none!(),       none!(),     none!(),      none!(),       none!(),       none!()     ],
		[ none!(),     none!(),       none!(),       none!(),     none!(),      none!(),       none!(),       none!()     ],
		[ mine!(Pawn), mine!(Pawn),   mine!(Pawn),   mine!(Pawn), mine!(Pawn),  mine!(Pawn),   mine!(Pawn),   mine!(Pawn) ],
		[ mine!(Rook), mine!(Knight), mine!(Bishop), mine!(King), mine!(Queen), mine!(Bishop), mine!(Knight), mine!(Rook) ],
	]
}
