use super::{
    defs::{SearchRefs, CHECK_TERMINATION, SEND_STATS},
    Search,
};
use crate::{
    defs::MAX_PLY,
    evaluation::{self, evaluate_position},
    movegen::defs::{Move, MoveList, MoveType, ShortMove},
};

impl Search {
    pub fn quiescent(
        mut alpha: i16,
        beta: i16,
        possible_moves: &mut Vec<Move>,
        refs: &mut SearchRefs,
    ) -> i16 {
        refs.search_info.nodes += 1;

        // evaluate and return
        if refs.search_info.ply == MAX_PLY {
            return evaluate_position(refs.board);
        }

        // Stand-pat
        let eval_score = evaluate_position(refs.board);
        if eval_score >= beta {
            return beta;
        }

        // Keep improving alpha score
        if eval_score > alpha {
            alpha = eval_score;
        }

        // Generate moves only with captures
        let mut move_list = MoveList::new();

        refs.move_generator
            .generate_moves(&refs.board, &mut move_list, MoveType::Capture);

        // Same as before make sure "best" moves are evaluated first
        Search::score_moves(&mut move_list, ShortMove::new(0), refs);

        for x in 0..move_list.len() {
            // Pick next move
            Search::swap_move(&mut move_list, x);

            let current_move = move_list.get_move(x);
            let is_legal = refs.board.make_move(current_move, refs.move_generator);

            if !is_legal {
                continue;
            }

            refs.search_info.ply += 1;

            let mut node_pv: Vec<Move> = Vec::new();

            let eval_score = Search::quiescent(-beta, -alpha, &mut node_pv, refs);

            // reset move on board
            refs.board.unmake();
            refs.search_info.ply -= 1;

            // if worse then beta (opp) then stop
            if eval_score >= beta {
                return beta;
            }

            if eval_score > alpha {
                alpha = eval_score;

                possible_moves.clear();
                possible_moves.push(current_move);
                possible_moves.append(&mut node_pv);
            }
        }

        alpha
    }
}
