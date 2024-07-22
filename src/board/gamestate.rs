use super::defs::{Pieces, PIECE_NAME, SQUARE_NAME};
use crate::movegen::defs::castling_as_string;
use crate::{
    defs::{self, Piece, Sides},
    movegen::defs::Move,
};

#[derive(Clone, Copy, Debug)]
pub struct GameState {
    pub active_color: u8,
    pub castling: u8,
    pub en_passant: Option<u8>,
    pub halfclock_move: u8,
    pub fullmove_number: u16,
    pub zobrist_key: u64,
    pub psqt: [i16; Sides::BOTH],
    pub material: [u16; Sides::BOTH],
    pub next_move: Move,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            active_color: 0,
            castling: 0,
            en_passant: None,
            halfclock_move: 0,
            fullmove_number: 0,
            zobrist_key: 0,
            psqt: [0; Sides::BOTH],
            material: [0; Sides::BOTH],
            next_move: Move::new(0),
        }
    }

    pub fn as_string(&self) -> String {
        let en_passant = if let Some(x) = self.en_passant {
            SQUARE_NAME[x as usize]
        } else {
            "-"
        };

        let promotion = if self.next_move.promoted() != Pieces::NONE {
            PIECE_NAME[self.next_move.promoted()]
        } else {
            ""
        };

        format!(
            "zk: {:x} ac: {} cperm: {} ep: {} hmc: {} fmn: {} mat: {}/{}, psqt: {}/{} next: {}{}{}",
            self.zobrist_key,
            self.active_color,
            castling_as_string(self.castling),
            en_passant,
            self.halfclock_move,
            self.fullmove_number,
            self.material[Sides::WHITE],
            self.material[Sides::BLACK],
            self.psqt[Sides::WHITE],
            self.psqt[Sides::BLACK],
            SQUARE_NAME[self.next_move.from()],
            SQUARE_NAME[self.next_move.to()],
            promotion
        )
    }
}
