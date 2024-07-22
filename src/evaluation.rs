use crate::{
    board::{defs::Pieces, Board},
    defs::Sides,
    evaluation::{defs::PIECE_VALUES, psqt::KING_EDGE},
};

pub mod defs;
pub mod material;
pub mod psqt;

pub fn evaluate_position(board: &Board) -> i16 {
    const PAWN_VALUE: i16 = PIECE_VALUES[Pieces::PAWN] as i16;

    let side = board.gamestate.active_color as usize;

    let w_material = board.gamestate.material[Sides::WHITE] as i16;
    let b_material = board.gamestate.material[Sides::BLACK] as i16;

    // base evaluation
    let mut eval = w_material - b_material;

    eval += board.gamestate.psqt[Sides::WHITE] - board.gamestate.psqt[Sides::BLACK];

    if w_material < PAWN_VALUE || b_material < PAWN_VALUE {
        let w_king_edge = KING_EDGE[board.king_square(Sides::WHITE)] as i16;
        let b_king_edge = KING_EDGE[board.king_square(Sides::BLACK)] as i16;

        eval += w_king_edge - b_king_edge;
    }

    eval = if side == Sides::WHITE { -eval } else { eval };

    eval
}
