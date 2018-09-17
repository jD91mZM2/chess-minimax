use crate::{
    board::Board,
    piece::PieceKind,
    Pos,
    Side
};
use std::sync::atomic::{AtomicBool, Ordering};

/// The result of a minimax session
pub struct MinimaxResult {
    pub score: i32,
    pub from: Pos,
    pub to: Pos
}

impl Board {
    /// Calculate the total score for a certain side
    pub fn score(&mut self, side: Side) -> i32 {
        let mut score = 0;
        let mut pieces = self.pieces(side);
        while let Some(pos) = pieces.next(&self) {
            // Material score
            let piece = self.get(pos).unwrap();
            score += piece.kind.worth() as i32 * 100;

            // Prioritize moves that reach a lot of position
            let mut moves = self.moves_for(pos);
            while moves.next(self).is_some() {
                score += 1;
            }
        }
        score
    }

    /// Return the best move according to a simple minimax algorithm
    pub fn minimax(&mut self, depth: u8, player: Side, exit: Option<&AtomicBool>) -> Option<MinimaxResult> {
        assert_ne!(depth, 0, "can't start minimax with 0 depth");
        self.minimax_inner(depth, player, player, exit, std::i32::MIN, std::i32::MAX)
    }
    fn minimax_inner(
        &mut self,
        depth: u8,
        original: Side,
        player: Side,
        exit: Option<&AtomicBool>,
        mut alpha: i32,
        mut beta: i32,
    ) -> Option<MinimaxResult> {
        let maximizing = original == player;
        let mut best: Option<MinimaxResult> = None;

        let mut pieces = self.pieces(player);
        'outer: while let Some(from) = pieces.next(self) {
            let mut moves = self.moves_for(from);
            while let Some(to) = moves.next(self) {
                let game_over = if maximizing { 99999 + depth as i32 } else { -99999 - depth as i32 };

                let score = if let Some(piece) = self.get(to).filter(|p| p.kind == PieceKind::King) {
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
                };

                if exit.map(|exit| exit.load(Ordering::SeqCst)).unwrap_or(false) {
                    return None;
                }

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
