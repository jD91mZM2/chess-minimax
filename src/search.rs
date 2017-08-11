use *;

const MAX_DEPTH: u8 = 4;

pub fn score(board: &Board) -> i32 {
	let mut score = 0;
	for line in board {
		for piece in line {
			if piece.is_mine() {
				score += piece.worth() as i32;
			} else {
				score -= piece.worth() as i32;
			}
		}
	}

	score
}
pub fn search(board: &mut Board, mine: bool, depth: u8, mut alpha: i32, mut beta: i32) -> (i32, (i8, i8), (i8, i8)) {
	let mut myking   = false;
	let mut yourking = false;
	for line in &*board {
		for piece in line {
			if let Piece::King(mine) = *piece {
				if mine {
					myking = true;
				} else {
					yourking = true;
				}
			}
		}
	}

	if !myking {
		// Play for as long as possible
		return (-(999 + depth as i32), (0, 0), (0, 0));
	} else if !yourking {
		// Play for as short as possible
		return (999 - depth as i32, (0, 0), (0, 0));
	}

	// I used to make my king worth 1000 and your king worth 100.
	// But then I realized:
	// If the game ended, don't go any further.

	if depth > MAX_DEPTH {
		return (score(board), (0, 0), (0, 0));
	}
	let possible = possible_moves(board, mine);

	let mut max_or_min = if mine { std::i32::MIN } else { std::i32::MAX };
	let mut selected   = ((0, 0), (0, 0));
	let mut found      = false;

	for (old, moves2) in possible {
		for new in &moves2 {
			let new = *new;
			let score;

			// It *could* only return old_to, but then
			// it wouldn't undo Pawn -> Queen.
			let (old_from, old_to, _) = board_move(board, old, new);

			score = search(board, !mine, depth + 1, alpha, beta).0;

			board_set(board, old, old_from);
			board_set(board, new, old_to);

			if (mine && score > max_or_min) || (!mine && score < max_or_min) {
				max_or_min = score;
				selected   = (old, new);
				found      = true;

				if mine && max_or_min > alpha {
					alpha = max_or_min;
				} else if !mine && max_or_min < beta {
					beta = max_or_min;
				}
				if beta <= alpha {
					break;
				}
			}
		}
	}

	if found {
		(max_or_min, selected.0, selected.1)
	} else {
		(score(board), (0, 0), (0, 0))
	}
}
