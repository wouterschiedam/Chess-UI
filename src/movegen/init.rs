use super::MoveGenerator;
use crate::board::defs::{Files, BB_FILES, BB_SQUARES};
use crate::movegen::magics::{Magic, BISHOP_MAGIC_NRS, ROOK_MAGIC_NRS};
use crate::movegen::{defs::*, BISHOP_TABLE_SIZE, ROOK_TABLE_SIZE};
use crate::{
    board::defs::{Pieces, RangeOf},
    defs::{Piece, Sides, EMPTY},
};

impl MoveGenerator {
    // King attacks table [square]
    pub fn init_king_attack(&mut self) {
        for square in RangeOf::SQUARES {
            let mut bitboard: u64 = 0;
            let mut attacks: u64 = 0;
            set_bit(&mut bitboard, square);

            if (bitboard >> 8) != 0 {
                attacks |= bitboard >> 8;
            }
            if (bitboard >> 9) & NOT_H_FILE != 0 {
                attacks |= bitboard >> 9;
            }
            if (bitboard >> 7) & NOT_A_FILE != 0 {
                attacks |= bitboard >> 7;
            }
            if (bitboard >> 1) & NOT_H_FILE != 0 {
                attacks |= bitboard >> 1;
            }
            if (bitboard << 8) != 0 {
                attacks |= bitboard << 8;
            }
            if (bitboard << 9) & NOT_A_FILE != 0 {
                attacks |= bitboard << 9;
            }
            if (bitboard << 7) & NOT_H_FILE != 0 {
                attacks |= bitboard << 7;
            }
            if (bitboard << 1) & NOT_A_FILE != 0 {
                attacks |= bitboard << 1;
            }

            self.king[square] = attacks;
        }
    }

    // Knight attacks table [square]
    pub fn init_knight_attack(&mut self) {
        for square in RangeOf::SQUARES {
            let mut bitboard: u64 = 0;
            let mut attacks: u64 = 0;
            set_bit(&mut bitboard, square);

            if ((bitboard >> 17) & NOT_H_FILE) != 0 {
                attacks |= bitboard >> 17;
            }
            if ((bitboard >> 15) & NOT_A_FILE) != 0 {
                attacks |= bitboard >> 15;
            }
            if ((bitboard >> 10) & NOT_HG_FILE) != 0 {
                attacks |= bitboard >> 10;
            }
            if ((bitboard >> 6) & NOT_AB_FILE) != 0 {
                attacks |= bitboard >> 6;
            }
            if ((bitboard << 17) & NOT_A_FILE) != 0 {
                attacks |= bitboard << 17;
            }
            if ((bitboard << 15) & NOT_H_FILE) != 0 {
                attacks |= bitboard << 15;
            }
            if ((bitboard << 10) & NOT_AB_FILE) != 0 {
                attacks |= bitboard << 10;
            }
            if ((bitboard << 6) & NOT_HG_FILE) != 0 {
                attacks |= bitboard << 6;
            }

            self.knight[square] = attacks;
        }
    }

    // Pawn attacks table [side][square]
    pub fn init_pawn_attack(&mut self) {
        for sq in RangeOf::SQUARES {
            let bb_square = BB_SQUARES[sq];
            let w = (bb_square & !BB_FILES[Files::A]) << 7 | (bb_square & !BB_FILES[Files::H]) << 9;
            let b = (bb_square & !BB_FILES[Files::A]) >> 9 | (bb_square & !BB_FILES[Files::H]) >> 7;
            self.pawns[Sides::WHITE][sq] = w;
            self.pawns[Sides::BLACK][sq] = b;
        }
    }

    pub fn init_magics(&mut self, piece: Piece) {
        // Check for valid piece
        let ok = piece == Pieces::ROOK || piece == Pieces::BISHOP;
        assert!(ok, "Illegal piece: {piece}");

        let is_rook = piece == Pieces::ROOK;

        let mut offset = 0;

        for square in RangeOf::SQUARES {
            let r_mask = MoveGenerator::rook_mask(square);
            let b_mask = MoveGenerator::bishop_mask(square);
            let mask = if is_rook { r_mask } else { b_mask };

            let bits = mask.count_ones();
            let permutations = 2u64.pow(bits); // amount of permutations
            let end = offset + permutations - 1;
            let blocker_boards = MoveGenerator::blocker_boards(mask);
            let r_ab = MoveGenerator::rook_attack(square, &blocker_boards);
            let b_ab = MoveGenerator::bishop_attack(square, &blocker_boards);
            let attack_boards = if is_rook { r_ab } else { b_ab };

            // generate magic numbers
            let mut magic: Magic = Default::default(); // New magic
            let r_magic_nr = ROOK_MAGIC_NRS[square];
            let b_magic_nr = BISHOP_MAGIC_NRS[square];

            // Set up the new magic with the values we already know.
            magic.mask = mask;
            magic.shift = (64 - bits) as u8;
            magic.offset = offset;
            magic.nr = if is_rook { r_magic_nr } else { b_magic_nr };

            for i in 0..permutations {
                let next = i as usize;
                let index = magic.get_index(blocker_boards[next]);
                let rook_table = &mut self.rook[..];
                let bishop_table = &mut self.bishop[..];
                let table = if is_rook { rook_table } else { bishop_table };

                if table[index] == EMPTY {
                    let fail_low = index < offset as usize;
                    let fail_high = index > end as usize;
                    assert!(!fail_low && !fail_high, "Indexing error. Error in Magics.");
                    table[index] = attack_boards[next];
                } else {
                    panic!("Attack table index not empty. Error in Magics.");
                }
            }

            // Store magics
            if is_rook {
                self.rook_magics[square] = magic;
            } else {
                self.bishop_magics[square] = magic;
            }

            // Do the next magic.
            offset += permutations;
        }

        // All permutations (blocker boards) should have been indexed.
        let r_ts = ROOK_TABLE_SIZE as u64;
        let b_ts = BISHOP_TABLE_SIZE as u64;
        let expectation = if is_rook { r_ts } else { b_ts };
        const ERROR: &str = "Initializing magics failed. Check magic numbers.";

        assert!(offset == expectation, "{}", ERROR);
    }
}
