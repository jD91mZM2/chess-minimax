use *;

const MAX_DEPTH: u8 = 3;

pub fn search(board: &mut Board, black: bool, depth: u8) -> (i32, (i8, i8), (i8, i8)) {
	if depth > MAX_DEPTH {
		return (0, (0, 0), (0, 0));
	}
	let possible = possible_moves(board, black);

	let mut scores = Vec::new();
	let mut moves: Vec<((i8, i8), (i8, i8))> = Vec::new();

	for ((x, y), moves2) in possible {
		for (new_x, new_y) in moves2 {
			let mut score;

			let old = board[new_y as usize][new_x as usize].clone();
			board[new_y as usize][new_x as usize] = board[y as usize][x as usize].clone();
			board[y as usize][x as usize] = Piece::Empty;

			let new_possible = possible_moves(board, black);
			if is_check(board, black, &new_possible).is_some() {
				score = if black { -10 } else { 10 };
			} else {
				score = search(board, !black, depth + 1).0;
				if !old.is_empty() {
					score = if black { 1 } else { -1 };
				}
			}

			// println!("Possible move:\n{}", board_string(&board));

			scores.push(score);
			moves.push(((x, y), (new_x, new_y)));

			board[y as usize][x as usize] = board[new_y as usize][new_x as usize].clone();
			board[new_y as usize][new_x as usize] = old;
		}
	}

	if black {
		let mut max = std::i32::MIN;
		let mut index = 0;
		for (i, score) in scores.iter().enumerate() {
			if *score > max {
				max = *score;
				index = i;
			}
		}

		(max, moves[index].0, moves[index].1)
	} else {
		let mut min = std::i32::MAX;
		let mut index = 0;
		for (i, score) in scores.iter().enumerate() {
			if *score < min {
				min = *score;
				index = i;
			}
		}

		(min, moves[index].0, moves[index].1)
	}
}
