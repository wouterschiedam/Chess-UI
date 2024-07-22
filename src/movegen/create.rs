use super::{defs::print_bitboard, MoveGenerator};
use crate::{
    board::{
        defs::{Direction, Files, Location, Ranks, BB_FILES, BB_RANKS, BB_SQUARES},
        Board,
    },
    defs::{Bitboard, Square},
};

pub type BlockerBoards = Vec<Bitboard>;
pub type AttackBoard = Vec<Bitboard>;

impl MoveGenerator {
    pub fn bishop_mask(square: usize) -> Bitboard {
        let location = Board::square_on_file_rank(square);
        let bb_edges = MoveGenerator::edges_without_piece(location);
        let bb_up_left = MoveGenerator::bb_ray(0, square, Direction::UpLeft);
        let bb_up_right = MoveGenerator::bb_ray(0, square, Direction::UpRight);
        let bb_down_right = MoveGenerator::bb_ray(0, square, Direction::DownRight);
        let bb_down_left = MoveGenerator::bb_ray(0, square, Direction::DownLeft);

        (bb_up_left | bb_up_right | bb_down_right | bb_down_left) & !bb_edges
    }

    fn edges_without_piece(location: Location) -> Bitboard {
        let bb_piece_file = BB_FILES[location.0 as usize];
        let bb_piece_rank = BB_RANKS[location.1 as usize];

        (BB_FILES[Files::A] & !bb_piece_file)
            | (BB_FILES[Files::H] & !bb_piece_file)
            | (BB_RANKS[Ranks::R1] & !bb_piece_rank)
            | (BB_RANKS[Ranks::R8] & !bb_piece_rank)
    }

    pub fn rook_mask(square: usize) -> Bitboard {
        let location = Board::square_on_file_rank(square);
        let bb_rook_square = BB_SQUARES[square];
        let bb_edges = MoveGenerator::edges_without_piece(location);
        let bb_mask = BB_FILES[location.0 as usize] | BB_RANKS[location.1 as usize];

        bb_mask & !bb_edges & !bb_rook_square
    }
    pub fn rook_attack(square: usize, blockers: &[Bitboard]) -> AttackBoard {
        // result attacks bitboard
        let mut bb_attack_boards: AttackBoard = Vec::new();
        for b in blockers.iter() {
            let bb_attacks = MoveGenerator::bb_ray(*b, square, Direction::Up)
                | MoveGenerator::bb_ray(*b, square, Direction::Right)
                | MoveGenerator::bb_ray(*b, square, Direction::Down)
                | MoveGenerator::bb_ray(*b, square, Direction::Left);
            bb_attack_boards.push(bb_attacks);
        }

        bb_attack_boards
    }

    // Same as the function above, but for the bishop.
    pub fn bishop_attack(square: Square, blockers: &[Bitboard]) -> AttackBoard {
        let mut bb_attack_boards: AttackBoard = Vec::new();

        for b in blockers.iter() {
            let bb_attacks = MoveGenerator::bb_ray(*b, square, Direction::UpLeft)
                | MoveGenerator::bb_ray(*b, square, Direction::UpRight)
                | MoveGenerator::bb_ray(*b, square, Direction::DownRight)
                | MoveGenerator::bb_ray(*b, square, Direction::DownLeft);
            bb_attack_boards.push(bb_attacks);
        }

        bb_attack_boards
    }
    pub fn blocker_boards(mask: Bitboard) -> BlockerBoards {
        // d is a copy of the input mask
        let d: Bitboard = mask;

        // Initialize the result vector
        let mut bb_blocker_boards: BlockerBoards = Vec::new();

        // n is the current subset of the input mask
        let mut n: Bitboard = 0;

        // Carry-Rippler method for generating subsets
        loop {
            // Add the current subset to the result vector
            bb_blocker_boards.push(n);

            // Update n using the Carry-Rippler method
            n = n.wrapping_sub(d) & d;

            // Break the loop if n becomes zero
            if n == 0 {
                break;
            }
        }

        // Return the vector of blocker boards
        bb_blocker_boards
    }

    pub fn bb_ray(bb_in: Bitboard, square: Square, direction: Direction) -> Bitboard {
        let mut file = Board::square_on_file_rank(square).0 as usize;
        let mut rank = Board::square_on_file_rank(square).1 as usize;
        let mut bb_square = BB_SQUARES[square];
        let mut bb_ray = 0;
        let mut done = false;
        while !done {
            done = true;
            match direction {
                Direction::Up => {
                    if rank != Ranks::R8 {
                        bb_square <<= 8;
                        bb_ray |= bb_square;
                        rank += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::Right => {
                    if file != Files::H {
                        bb_square <<= 1;
                        bb_ray |= bb_square;
                        file += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::Down => {
                    if rank != Ranks::R1 {
                        bb_square >>= 8;
                        bb_ray |= bb_square;
                        rank -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::Left => {
                    if file != Files::A {
                        bb_square >>= 1;
                        bb_ray |= bb_square;
                        file -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::UpLeft => {
                    if (rank != Ranks::R8) && (file != Files::A) {
                        bb_square <<= 7;
                        bb_ray |= bb_square;
                        rank += 1;
                        file -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::UpRight => {
                    if (rank != Ranks::R8) && (file != Files::H) {
                        bb_square <<= 9;
                        bb_ray |= bb_square;
                        rank += 1;
                        file += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::DownRight => {
                    if (rank != Ranks::R1) && (file != Files::H) {
                        bb_square >>= 7;
                        bb_ray |= bb_square;
                        rank -= 1;
                        file += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::DownLeft => {
                    if (rank != Ranks::R1) && (file != Files::A) {
                        bb_square >>= 9;
                        bb_ray |= bb_square;
                        rank -= 1;
                        file -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
            };
        }
        bb_ray
    }
}
