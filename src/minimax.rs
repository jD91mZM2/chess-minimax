use crate::{
    board::Board,
    piece::PieceKind,
    utils::stackvec::StackVec,
    Pos,
    Side
};
use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread
};

/// The result of a minimax session
pub struct MinimaxResult {
    pub score: i16,
    pub from: Pos,
    pub to: Pos
}

impl Board {
    pub fn minimax(&mut self, depth: u8, player: Side, exit: Option<&AtomicBool>) -> Option<MinimaxResult> {
        assert_ne!(depth, 0, "can't start minimax with 0 depth");
        self.minimax_inner(depth, player, player, exit, std::i16::MIN, std::i16::MAX)
    }
    pub fn minimax_parallel(&mut self, depth: u8, player: Side, exit: Option<&Arc<AtomicBool>>)
        -> Option<MinimaxResult>
    {
        const WORKERS: usize = 4;

        assert!(depth > 1, "can't start a parallel minimax with 1 or less depth");

        let mut total_pieces = 0;

        let mut pieces = self.pieces(player);
        while pieces.next(self).is_some() {
            total_pieces += 1;
        }

        let pieces_per_worker = total_pieces / WORKERS;

        let mut todo = Vec::with_capacity(pieces_per_worker);
        let mut threads: StackVec<[_; WORKERS]> = StackVec::new();

        let mut pieces = self.pieces(player);
        while let Some(piece) = pieces.next(self) {
            if todo.len() >= pieces_per_worker {
                threads.push(self.minimax_worker(depth, player, exit, todo.clone()));
                todo.clear();
            }
            todo.push(piece);
        }

        if !todo.is_empty() {
            threads.push(self.minimax_worker(depth, player, exit, todo));
        }

        if exit.map(|b| b.load(Ordering::SeqCst)).unwrap_or(false) {
            return None;
        }

        let mut best: Option<MinimaxResult> = None;

        for thread in threads {
            let result = thread.join().unwrap();
            if let Some(result) = result {
                if best.as_ref().map(|best| result.score > best.score).unwrap_or(true) {
                    best = Some(result);
                }
            }
        }

        if exit.map(|exit| exit.load(Ordering::SeqCst)).unwrap_or(false) {
            return None;
        }

        best
    }
    fn minimax_worker(
        &mut self,
        depth: u8,
        player: Side,
        exit: Option<&Arc<AtomicBool>>,
        pieces: Vec<Pos>
    ) -> thread::JoinHandle<Option<MinimaxResult>> {
        let mut board = self.clone();
        let exit = exit.map(|exit| Arc::clone(&exit));
        thread::spawn(move || {
            let mut best: Option<MinimaxResult> = None;
            let exit = exit.as_ref().map(|exit| &**exit);

            for from in pieces {
                let mut moves = board.moves_for(from);
                while let Some(to) = moves.next(&mut board) {
                    if exit.map(|exit| exit.load(Ordering::SeqCst)).unwrap_or(false) {
                        return None;
                    }

                    let score = board.minimax_try(depth, player, player, exit, std::i16::MIN, std::i16::MAX, from, to);

                    if best.as_ref().map(|best| score > best.score).unwrap_or(true) {
                        best = Some(MinimaxResult {
                            score,
                            from,
                            to
                        });
                    }
                }
            }

            best
        })
    }
    fn minimax_try(
        &mut self,
        depth: u8,
        original: Side,
        player: Side,
        exit: Option<&AtomicBool>,
        alpha: i16,
        beta: i16,
        from: Pos,
        to: Pos
    ) -> i16 {
        let game_over = if original == player { 999 + depth as i16 } else { -999 - depth as i16 };

        if let Some(piece) = self.get(to).filter(|p| p.kind == PieceKind::King) {
            assert!(player != piece.side);
            game_over
        } else {
            // Apply move
            let undo = self.move_(from, to);

            let score = if depth == 1 {
                self.score(original) - self.score(!original)
            } else {
                self.minimax_inner(depth - 1, original, !player, exit, alpha, beta)
                    .map(|s| s.score)
                    .unwrap_or(game_over)
            };

            // Undo move
            self.undo(undo);

            score
        }
    }
    fn minimax_inner(
        &mut self,
        depth: u8,
        original: Side,
        player: Side,
        exit: Option<&AtomicBool>,
        mut alpha: i16,
        mut beta: i16,
    ) -> Option<MinimaxResult> {
        let maximizing = original == player;
        let mut best: Option<MinimaxResult> = None;

        let mut pieces = self.pieces(player);
        'outer: while let Some(from) = pieces.next(self) {
            let mut moves = self.moves_for(from);
            while let Some(to) = moves.next(self) {
                if exit.map(|exit| exit.load(Ordering::SeqCst)).unwrap_or(false) {
                    return None;
                }

                let score = self.minimax_try(depth, original, player, exit, alpha, beta, from, to);

                let this = Some(MinimaxResult {
                    score,
                    from,
                    to
                });
                if maximizing && best.as_ref().map(|best| score > best.score).unwrap_or(true) {
                    best = this;
                    if score > alpha {
                        // alpha holds upper half of possible scores
                        alpha = score;
                    }
                } else if !maximizing && best.as_ref().map(|best| score < best.score).unwrap_or(true) {
                    best = this;
                    if score < beta {
                        // beta holds lower half of possible scores
                        beta = score;
                    }
                }
                if alpha >= beta {
                    // This node will not be chosen by the parent node, because
                    // it has a worse value than a previous node.
                    // If this is just as confusing to you as it is to me,
                    // https://youtu.be/xBXHtz4Gbdo might be a good resource.
                    break 'outer;
                }
            }
        }

        best
    }
}
