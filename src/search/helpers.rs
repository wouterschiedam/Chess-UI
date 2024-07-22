use crate::{
    board::{defs::Pieces, Board},
    defs::{Sides, MAX_MOVE_RULE},
};

use super::{defs::SearchRefs, Search};

impl Search {
    pub fn is_draw(refs: &SearchRefs) -> bool {
        let max_move_rule = refs.board.gamestate.halfclock_move >= MAX_MOVE_RULE;

        // Check for max game_rule | insufficiant material | repetition
        max_move_rule || Search::insufficiant_material(refs) || Search::is_repition(refs.board) > 0
    }

    pub fn is_repition(board: &Board) -> u8 {
        let mut count = 0;
        let mut stop = false;
        let mut x = board.history.len() - 1;

        while x != 0 && !stop {
            let historic = board.history.get_ref(x);

            // if zobrist key are the same we found repetition
            if historic.zobrist_key == board.gamestate.zobrist_key {
                count += 1;
            }

            // if hcm is 0 it is because of a caputre or pawn move so this position cant have
            // existed before
            stop = historic.halfclock_move == 0;

            x -= 1;
        }

        count
    }

    // This function calculates the number of nodes per second.
    pub fn nodes_per_second(nodes: usize, msecs: u128) -> usize {
        let mut nps: usize = 0;
        let seconds = msecs as f64 / 1000f64;
        if seconds > 0f64 {
            nps = (nodes as f64 / seconds).round() as usize;
        }
        // if nps == 0 {
        //     nps = 1;
        // }
        nps
    }
}

#[rustfmt::skip]
impl Search {
    pub fn insufficiant_material(refs: &SearchRefs) -> bool {
        // It's not a draw if: ...there are still pawns.
        let w_p = refs.board.get_pieces(Pieces::PAWN, Sides::WHITE).count_ones() > 0;     
        let b_p = refs.board.get_pieces(Pieces::PAWN, Sides::BLACK).count_ones() > 0;        
        // ...there's a major piece on the board.
        let w_q = refs.board.get_pieces(Pieces::QUEEN, Sides::WHITE).count_ones() > 0;
        let b_q = refs.board.get_pieces(Pieces::QUEEN, Sides::BLACK).count_ones() > 0;
        let w_r = refs.board.get_pieces(Pieces::ROOK, Sides::WHITE).count_ones() > 0;
        let b_r = refs.board.get_pieces(Pieces::ROOK, Sides::BLACK).count_ones() > 0;
        // ...or two bishops for one side.
        let w_b = refs.board.get_pieces(Pieces::BISHOP, Sides::WHITE).count_ones() > 1;
        let b_b = refs.board.get_pieces(Pieces::BISHOP, Sides::BLACK).count_ones() > 1;
        // ... or a bishop+knight for at least one side.
        let w_bn =
            refs.board.get_pieces(Pieces::BISHOP, Sides::WHITE).count_ones() > 0 &&
            refs.board.get_pieces(Pieces::KNIGHT, Sides::WHITE).count_ones() > 0;
        let b_bn =
            refs.board.get_pieces(Pieces::BISHOP, Sides::BLACK).count_ones() > 0 &&
            refs.board.get_pieces(Pieces::KNIGHT, Sides::BLACK).count_ones() > 0;
         
        // If one of the conditions above is true, we still have enough
        // material for checkmate, so insufficient_material returns false.
        !(w_p || b_p || w_q || b_q || w_r || b_r || w_b || b_b ||  w_bn || b_bn)
    }
}
