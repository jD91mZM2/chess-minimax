use *;

const MAX_DEPTH: u8 = 3;

pub fn search(board: &mut Board, black: bool, depth: u8) -> (i32, (i8, i8), (i8, i8)) {
	if depth > MAX_DEPTH {
		let mut score = 0;
		for line in board {
			for piece in line {
				score += piece.worth();
			}
		}
		return (score as i32, (0, 0), (0, 0));
	}
	let possible = possible_moves(board, black);

	let mut max_or_min = if black { std::i32::MIN } else { std::i32::MAX };
	let mut selected   = ((0, 0), (0, 0));
	let mut found      = false;

	for ((x, y), moves2) in possible {
		for (new_x, new_y) in moves2 {
			let mut score;

			let old = board[new_y as usize][new_x as usize];
			board[new_y as usize][new_x as usize] = board[y as usize][x as usize];
			board[y as usize][x as usize] = Piece::Empty;

			macro_rules! repair {
				() => {
					board[y as usize][x as usize] = board[new_y as usize][new_x as usize];
					board[new_y as usize][new_x as usize] = old;
				}
			}

			if is_check(board, black, &possible_moves(board, !black)) {
				repair!();
				continue;
			}
			score = search(board, !black, depth + 1).0;

			// println!("Possible move:\n{}", board_string(&board));

			score = if black { score } else { -score };
			if (black && score > max_or_min) || (!black && score < max_or_min) {
				max_or_min = score;
				selected   = ((x, y), (new_x, new_y));
				found      = true;
			}

			repair!();
		}
	}

	if found {
		(max_or_min, selected.0, selected.1)
	} else {
		(0, (0, 0), (0, 0))
	}
}
