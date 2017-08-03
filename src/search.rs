use *;

const MAX_DEPTH: u8 = 3;

pub fn search(board: &mut Board, black: bool, score: &mut i32, depth: u8) -> ((i8, i8), (i8, i8)) {
	if depth > MAX_DEPTH {
		return ((0, 0), (0, 0));
	}
	println!("Searching. Depth: {}", depth);

	let possible = possible_moves(board, black);

	let mut highest = ((0, 0), (0, 0));
	let mut max = None;

	for ((x, y), moves) in possible {
		for (new_x, new_y) in moves {
			let mut score = *score;

			let old = board[new_y as usize][new_x as usize].clone();
			board[new_y as usize][new_x as usize] = board[y as usize][x as usize].clone();
			board[y as usize][x as usize] = Piece::Empty;

			let new_possible = possible_moves(board, black);
			if is_check(board, black, &new_possible).is_some() {
				score += if black { -10 } else { 10 }
			} else {
				search(board, !black, &mut score, depth + 1);
			}
			if !old.is_empty() {
				score += if black { 1 } else { -1 }
			}

			if max.is_none() || (black && score > max.unwrap()) || (!black && score < max.unwrap() && score != 0) {
				max = Some(score);
				highest = ((x, y), (new_x, new_y));
			}

			board[y as usize][x as usize] = board[new_y as usize][new_x as usize].clone();
			board[new_y as usize][new_x as usize] = old;
		}
	}

	if black {
		println!("Black Score: {}", max.unwrap());
	} else {
		println!("White Score: {}", max.unwrap());
	}

	*score = max.unwrap();
	highest
}
