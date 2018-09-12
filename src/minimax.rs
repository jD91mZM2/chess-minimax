use crate::{
    board::Board,
    piece::PieceKind,
    Pos,
    Side
};

/// The result of a minimax session
pub struct MinimaxResult {
    pub score: i16,
    pub from: Pos,
    pub to: Pos
}

impl Board {
    pub fn minimax(&mut self, depth: u8, player: Side) -> Option<MinimaxResult> {
        assert_ne!(depth, 0, "can't start minimax with 0 depth");
        self.minimax_inner(depth, player, player, std::i32::MIN, std::i32::MAX)
    }
    fn minimax_inner(
        &mut self,
        depth: u8,
        original: Side,
        player: Side,
        _alpha: i32,
        _beta: i32
    ) -> Option<MinimaxResult> {
        let maximizing = original == player;
        let mut best = None;

        let mut pieces = self.pieces(player);
        while let Some(from) = pieces.next(&self) {
            let mut moves = self.moves_for(from);
            while let Some(to) = moves.next(&self) {
                let score = if let Some(piece) = self.get(to).filter(|p| p.kind == PieceKind::King) {
                    if original == piece.side {
                        -999 - depth as i16
                    } else {
                        999 + depth as i16
                    }
                } else {
                    // Apply move
                    let undo = self.move_(from, to);

                    let score = if depth == 1 {
                        self.score(original) - self.score(!original)
                    } else {
                        self.minimax_inner(depth - 1, player, !player, std::i32::MIN, std::i32::MAX)
                            .map(|s| s.score)
                            .unwrap_or(999 + depth as i16)
                    };

                    // Undo move
                    self.undo(undo);

                    score
                };

                //if best.as_ref().map(|best: &MinimaxResult| maximizing == (score > best.score)).unwrap_or(true) {
                if (maximizing && best.as_ref().map(|best: &MinimaxResult| score > best.score).unwrap_or(true))
                    || (!maximizing && best.as_ref().map(|best: &MinimaxResult| score < best.score).unwrap_or(true)){
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
