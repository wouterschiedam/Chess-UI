use super::{
    defs::{SearchRefs, SearchReport, SearchResult, SearchSummary, INF},
    Search,
};
use crate::{
    defs::{Sides, MAX_PLY},
    engine::defs::Information,
    movegen::defs::{print_bitboard, Move},
};

// Actual search routine
impl Search {
    pub fn search_routine(refs: &mut SearchRefs) -> SearchResult {
        let mut depth = 1;
        let mut best_move = Move::new(0);
        let mut root_pv: Vec<Move> = Vec::new();
        let mut stop = false;

        let alpha: i16 = -INF;
        let beta: i16 = INF;
        refs.search_info.start_timer();
        while (depth <= MAX_PLY) && (depth <= refs.search_params.depth) && !stop {
            // set current depth
            refs.search_info.depth = depth;
            // get eval for position
            let eval = Search::alpha_beta(depth, alpha, beta, &mut root_pv, refs);

            // if not interupted
            if !refs.search_info.interupted() {
                // save best move
                if !root_pv.is_empty() {
                    best_move = root_pv[0];
                }

                // Create summary of search
                let elapsed = refs.search_info.time_elapsed();
                let nodes = refs.search_info.nodes;
                let summary = SearchSummary {
                    depth,
                    seldepth: refs.search_info.seldepth,
                    time: elapsed,
                    cp: eval,
                    mate: 0,
                    nodes,
                    nps: Search::nodes_per_second(nodes, elapsed),
                    pv: root_pv.clone(),
                };

                // println!("{:?}", &summary);
                let report = SearchReport::SearchSummary(summary);
                let information = Information::Search(report);
                refs.report_tx
                    .send(information)
                    .expect("Couldnt send info to info channel");

                depth += 1;
            }

            // Stop deepening the search if the current depth was
            // interrupted, or if the time is up.
            stop = refs.search_info.interupted();
        }
        // refs.board.make_move(best_move, refs.move_generator);
        (best_move, refs.search_info.terminated)
    }
}
