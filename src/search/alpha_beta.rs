use crate::movegen::defs::{movelist, Move, MoveList, MoveType, ShortMove};

use super::{
    defs::{SearchRefs, CHECKMATE, CHECK_TERMINATION, DRAW, INF, SEND_STATS, STALEMATE},
    Search,
};

impl Search {
    pub fn alpha_beta(
        mut depth: i8,
        mut alpha: i16,
        beta: i16,
        possible_moves: &mut Vec<Move>,
        refs: &mut SearchRefs,
    ) -> i16 {
        // if refs.search_info.nodes & CHECK_TERMINATION == 0 {
        //     Search::check_termination();
        // }
        let mut pvs = false; // Principal variation search

        // First check if we are in check
        let is_check = refs.move_generator.square_attacked(
            refs.board,
            refs.board.side_to_not_move(),
            refs.board.king_square(refs.board.side_to_move()),
        );

        // Go deeper in depth to look for the best option to get out of the check
        if is_check {
            depth += 1;
        }

        if depth == 0 {
            // We are at top node evaluate the position and return the move
            return Search::quiescent(alpha, beta, possible_moves, refs);
        }

        // Add node we searched
        refs.search_info.nodes += 1;

        // Start searching //
        let mut legal_moves = 0;
        let mut move_list = MoveList::new();

        refs.move_generator
            .generate_moves(&refs.board, &mut move_list, MoveType::All);
        // First search "best" moves
        Search::score_moves(&mut move_list, ShortMove::new(0), refs);
        // Set best possible value to worst
        let mut best_eval_score = -INF;

        // hold the best move in var
        let mut best_possible_move: ShortMove = ShortMove::new(0);
        // iterate over moves
        for x in 0..move_list.len() {
            // evaluate best moves first based on score_moves()
            Search::swap_move(&mut move_list, x);

            let current_move = move_list.get_move(x);
            let is_legal = refs.board.make_move(current_move, refs.move_generator);

            // if not legal skip and move on
            if !is_legal {
                continue;
            }

            legal_moves += 1;
            refs.search_info.ply += 1;

            let mut node_pv: Vec<Move> = Vec::new();
            let mut eval_score = 0;

            // Check if game is a draw if not start searching
            if !Search::is_draw(refs) {
                if pvs {
                    eval_score =
                        -Search::alpha_beta(depth - 1, -alpha - 1, -alpha, &mut node_pv, refs);

                    // check pvs
                    if eval_score > alpha && eval_score < beta {
                        eval_score =
                            -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                    }
                } else {
                    eval_score = -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                }
            }

            refs.board.unmake();
            refs.search_info.ply -= 1;

            // eval_score is better than the best we found so far, so we
            // save a new best_move that'll go into the hash table.
            if eval_score > best_eval_score {
                best_eval_score = eval_score;
                best_possible_move = current_move.to_short_move();
            }

            // found better move
            if eval_score > alpha {
                alpha = eval_score;

                pvs = true;

                possible_moves.clear();
                possible_moves.push(current_move);
                possible_moves.append(&mut node_pv);
            }
        }

        // check if moves are possible
        // otherwise checkmate or stalemate
        if legal_moves == 0 {
            if is_check {
                return -CHECKMATE + (refs.search_info.ply as i16);
            } else {
                return -STALEMATE;
            }
        }

        alpha
    }
}
