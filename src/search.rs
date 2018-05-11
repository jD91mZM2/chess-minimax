use *;
#[cfg(feature = "threaded")]
use std::thread;
#[cfg(feature = "threaded")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "threaded")]
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering as AtomicOrdering};

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
pub fn start_search(board: &mut Board) -> (i32, Pos, Pos) {
    #[cfg(not(feature = "threaded"))]
    return search(board, true, 0, std::i32::MIN, std::i32::MAX);
    #[cfg(feature = "threaded")]
    {
        let possible = possible_moves(board, true);

        let found      = Arc::new(AtomicBool::new(false));
        let max_or_min = Arc::new(AtomicI32::new(std::i32::MIN));
        let selected   = Arc::new(Mutex::new(((0, 0), (0, 0))));

        let mut threads = Vec::new();

        for (old, moves2) in possible {
            for new in &moves2 {
                let new = *new;
                let mut board = board.clone();
                let mut max_or_min = max_or_min.clone();
                let mut selected   = selected.clone();
                let mut found      = found.clone();

                threads.push(thread::spawn(move || {
                    let (diff, _) = board_move(&mut board, old, new);
                    let score = search(&mut board, false, 1, std::i32::MIN, std::i32::MAX).0;

                    board_apply(&mut board, diff); // undo

                    if score > max_or_min.load(AtomicOrdering::Relaxed) {
                        max_or_min.store(score, AtomicOrdering::Relaxed);
                        *selected.lock().unwrap() = (old, new);
                        found.store(true, AtomicOrdering::Relaxed);
                    }
                }));
            }
        }

        for thread in threads {
            thread.join().unwrap();
        }

        let found      = found.load(AtomicOrdering::Relaxed);
        let max_or_min = max_or_min.load(AtomicOrdering::Relaxed);
        let selected   = selected.lock().unwrap();

        if found {
            (max_or_min, selected.0, selected.1)
        } else {
            (score(board), (0, 0), (0, 0))
        }
    }
}

pub fn search(board: &mut Board, mine: bool, depth: u8, mut alpha: i32, mut beta: i32) -> (i32, Pos, Pos) {
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

    let mut found      = false;
    let mut max_or_min = if mine { std::i32::MIN } else { std::i32::MAX };
    let mut selected   = ((0, 0), (0, 0));

    for (old, moves2) in possible {
        for new in &moves2 {
            let new = *new;

            let (diff, _) = board_move(board, old, new);
            let score = search(board, !mine, depth + 1, alpha, beta).0;

            board_apply(board, diff); // undo

            if (mine && score > max_or_min) || (!mine && score < max_or_min) {
                found      = true;
                max_or_min = score;
                selected   = (old, new);

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
