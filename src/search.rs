use *;

const MAX_DEPTH: u8 = 4;

pub fn score(board: &Board, black: bool) -> i32 {
	let mut score = 0;
	for line in board {
		for piece in line {
			if piece.is_black() == black {
				if let Piece::King(_) = *piece {
					// Because otherwise taking the other king is more important than actually moving away.
					// This is a cheap trick to make the king the most important character to protect.
					score += 1000;
				} else {
					score += piece.worth() as i32;
				}
			} else {
				score -= piece.worth() as i32;
			}
		}
	}

	if black { score } else { -score }
}
pub fn search(board: &mut Board, black: bool, depth: u8) -> (i32, (i8, i8), (i8, i8)) {
	if depth > MAX_DEPTH {
		let mut black = black; // Don't wanna make entire variable mutable for the entire function.
		if MAX_DEPTH % 2 == 0 {
			black = !black;
		}

		return (score(board, black), (0, 0), (0, 0));
	}
	let possible = possible_moves(board, black);

	let mut max_or_min = if black { std::i32::MIN } else { std::i32::MAX };
	let mut selected   = ((0, 0), (0, 0));
	let mut found      = false;

	for (old, moves2) in possible {
		for new in moves2 {
			let score;

			// It could only return old_to, but then
			// it wouldn't undo Pawn -> Queen.
			let (old_from, old_to) = board_move(board, old, new);

			score = search(board, !black, depth + 1).0;

			// println!("Possible move:\n{}", board_string(&board));

			board_set(board, old, old_from);
			board_set(board, new, old_to);

			if (black && score > max_or_min) || (!black && score < max_or_min) {
				max_or_min = score;
				selected   = (old, new);
				found      = true;
			}
		}
	}

	if found {
		(max_or_min, selected.0, selected.1)
	} else {
		(0, (0, 0), (0, 0))
	}
}
