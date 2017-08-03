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
			} else if is_check(board, !black, &possible_moves(board, black)) {
				// Ugh, try every possible move to be 100% if it's checkmate.
				let mut mate = true;
				for ((x, y), moves3) in possible_moves(board, !black) {
					for (new_x, new_y) in moves3 {
						let old = board[new_y as usize][new_x as usize];
						board[new_y as usize][new_x as usize] = board[y as usize][x as usize];
						board[y as usize][x as usize] = Piece::Empty;

						mate = !is_check(board, !black, &possible_moves(board, black));

						board[y as usize][x as usize] = board[new_y as usize][new_x as usize];
						board[new_y as usize][new_x as usize] = old;

						if !mate {
							break;
						}
					}
				}

				if mate {
					score = 100;
				} else {
					score = 1;
				}
			} else {
				score = search(board, !black, depth + 1).0;
				score += old.worth() as i32;
			}

			// println!("Possible move:\n{}", board_string(&board));

			scores.push(if black { score } else { -score });
			moves.push(((x, y), (new_x, new_y)));

			repair!();
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
