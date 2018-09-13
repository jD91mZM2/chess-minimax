use crate::{
    board::Board,
    piece::PieceKind,
    Pos,
    Side
};
use std::sync::atomic::{AtomicBool, Ordering};

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
    fn minimax_inner(
        &mut self,
        depth: u8,
        original: Side,
        player: Side,
        exit: Option<&AtomicBool>,
        _alpha: i16,
        _beta: i16,
    ) -> Option<MinimaxResult> {
        let maximizing = original == player;
        let mut best = None;

        let mut pieces = self.pieces(player);
        while let Some(from) = pieces.next(&self) {
            let mut moves = self.moves_for(from);
            while let Some(to) = moves.next(&self) {
                let game_over = if maximizing { 999 + depth as i16 } else { -999 - depth as i16 };

                let score = if let Some(piece) = self.get(to).filter(|p| p.kind == PieceKind::King) {
                    assert!(player != piece.side);
                    game_over
                } else {
                    // Apply move
                    let undo = self.move_(from, to);

                    let score = if depth == 1 {
                        self.score(original) - self.score(!original) +
                            if maximizing {
                                depth as i16
                            } else {
                                -(depth as i16)
                            }
                    } else {
                        self.minimax_inner(depth - 1, original, !player, exit, std::i16::MIN, std::i16::MAX)
                            .map(|s| s.score)
                            .unwrap_or(game_over)
                    };

                    // Undo move
                    self.undo(undo);

                    if exit.map(|exit| exit.load(Ordering::SeqCst)).unwrap_or(false) {
                        return None;
                    }

                    score
                };

                if best.as_ref().map(|best: &MinimaxResult| maximizing == (score > best.score)).unwrap_or(true) {
                    best = Some(MinimaxResult {
                        score,
                        from,
                        to
                    });
                }
            }
        }

        best
    }
}