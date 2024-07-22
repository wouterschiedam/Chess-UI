use super::{
    defs::{SearchRefs, MAX_KILLER_MOVES},
    Search,
};
use crate::{
    board::defs::Pieces,
    defs::NrOf,
    movegen::defs::{MoveList, ShortMove},
};

const MVV_LVA_OFFSET: u32 = u32::MAX - 256;
const TTMOVE_SORT_VALUE: u32 = 60;
const KILLER_VALUE: u32 = 10;

// MVV_VLA[victim][attacker]
pub const MVV_LVA: [[u16; NrOf::PIECE_TYPES + 1]; NrOf::PIECE_TYPES + 1] = [
    [0, 0, 0, 0, 0, 0, 0],       // victim K, attacker K, Q, R, B, N, P, None
    [50, 51, 52, 53, 54, 55, 0], // victim Q, attacker K, Q, R, B, N, P, None
    [40, 41, 42, 43, 44, 45, 0], // victim R, attacker K, Q, R, B, N, P, None
    [30, 31, 32, 33, 34, 35, 0], // victim B, attacker K, Q, R, B, N, P, None
    [20, 21, 22, 23, 24, 25, 0], // victim K, attacker K, Q, R, B, N, P, None
    [10, 11, 12, 13, 14, 15, 0], // victim P, attacker K, Q, R, B, N, P, None
    [0, 0, 0, 0, 0, 0, 0],       // victim None, attacker K, Q, R, B, N, P, None
];

impl Search {
    pub fn score_moves(move_list: &mut MoveList, short_move: ShortMove, refs: &SearchRefs) {
        for x in 0..move_list.len() {
            let t_move = move_list.get_mut_move(x);
            let mut value: u32 = 0;
            // Sort moves // TT first, Then Capture, then quiet moves
            if t_move.get_move() == short_move.get_move() {
                value = MVV_LVA_OFFSET + TTMOVE_SORT_VALUE;
            } else if t_move.captured() != Pieces::NONE {
                // Set capture higher then MVV_LVA offset
                value = MVV_LVA_OFFSET + MVV_LVA[t_move.captured()][t_move.piece()] as u32;
            } else {
                let ply = refs.search_info.ply as usize;
                let mut n = 0;
                while n < MAX_KILLER_MOVES && value == 0 {
                    let killer = refs.search_info.killer_moves[ply][n];
                    if t_move.get_move() == killer.get_move() {
                        value = MVV_LVA_OFFSET - ((x as u32 + 1) & KILLER_VALUE);
                    }
                    n += 1;
                }
            }

            t_move.set_sort_score(value);
        }
    }

    // do swapping in movelist
    pub fn swap_move(move_list: &mut MoveList, start_index: u8) {
        for x in (start_index + 1)..move_list.len() {
            if move_list.get_move(x).get_sort_score()
                > move_list.get_move(start_index).get_sort_score()
            {
                move_list.swap(start_index as usize, x as usize);
            }
        }
    }
}
