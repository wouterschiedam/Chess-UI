use crate::defs::{NrOf, Piece, Side, Sides, Square, EMPTY};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;

/* Random number for all sides for all pieces on all squares */
// [side][square][piece]
type PieceRandoms = [[[u64; NrOf::SQUARES]; NrOf::PIECE_TYPES]; Sides::BOTH];
type CastlingRandoms = [u64; NrOf::CASTLING_PERMISSIONS];
type SideRandoms = [u64; Sides::BOTH];
type EpRandoms = [u64; NrOf::SQUARES + 1];

pub type ZobristKey = u64;

// 256 bit (8 bits x 32) seed
const RNG_SEED: [u8; 32] = [125; 32];
#[derive(Debug)]
pub struct ZobristRandoms {
    rnd_pieces: PieceRandoms,
    rnd_castling: CastlingRandoms,
    rnd_side: SideRandoms,
    rnd_en_passant: EpRandoms,
}

impl ZobristRandoms {
    pub fn new() -> Self {
        let mut rand = ChaChaRng::from_seed(RNG_SEED);

        let mut zobrist_random = Self {
            rnd_pieces: [[[EMPTY; NrOf::SQUARES]; NrOf::PIECE_TYPES]; Sides::BOTH],
            rnd_castling: [EMPTY; NrOf::CASTLING_PERMISSIONS],
            rnd_side: [EMPTY; Sides::BOTH],
            rnd_en_passant: [EMPTY; NrOf::SQUARES + 1],
        };

        zobrist_random.rnd_pieces.iter_mut().for_each(|side| {
            side.iter_mut().for_each(|piece| {
                piece
                    .iter_mut()
                    .for_each(|square| *square = rand.gen::<u64>());
            })
        });

        zobrist_random
            .rnd_castling
            .iter_mut()
            .for_each(|permission| *permission = rand.gen::<u64>());

        zobrist_random
            .rnd_side
            .iter_mut()
            .for_each(|side| *side = rand.gen::<u64>());

        zobrist_random
            .rnd_en_passant
            .iter_mut()
            .for_each(|ep| *ep = rand.gen::<u64>());

        zobrist_random
    }

    pub fn piece(&self, side: Side, square: Square, piece: Piece) -> ZobristKey {
        self.rnd_pieces[side][piece][square]
    }

    pub fn castling(&self, castling_perm: u8) -> ZobristKey {
        self.rnd_castling[castling_perm as usize]
    }

    pub fn side(&self, side: Side) -> ZobristKey {
        self.rnd_side[side]
    }

    pub fn en_passant(&self, en_passant: Option<u8>) -> ZobristKey {
        match en_passant {
            Some(ep) => self.rnd_en_passant[ep as usize],
            None => self.rnd_en_passant[NrOf::SQUARES],
        }
    }
}
