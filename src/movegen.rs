mod create;
pub mod defs;
mod init;
mod magics;
mod movelist;

use std::time::{Duration, Instant};

use crate::{
    board::{
        defs::{Pieces, Squares, BB_RANKS, BB_SQUARES, PIECE_CHAR_CAPS, PIECE_NAME, SQUARE_NAME},
        Board,
    },
    defs::{Bitboard, Castling, NrOf, Piece, Side, Sides, Square, EMPTY},
    extra::bits::{self, next},
};

use crate::movegen::magics::Magic;

use self::defs::{print_bitboard, Move, MoveList, MoveType, Shift};

pub const PROMOTION_PIECES: [usize; 4] =
    [Pieces::QUEEN, Pieces::ROOK, Pieces::KNIGHT, Pieces::BISHOP];

pub const ROOK_TABLE_SIZE: usize = 102_400; // Total permutations of all rook blocker boards.
pub const BISHOP_TABLE_SIZE: usize = 5_248; // Total permutations of all bishop blocker boards.
pub struct MoveGenerator {
    pub king: [Bitboard; NrOf::SQUARES],
    pub knight: [Bitboard; NrOf::SQUARES],
    pub pawns: [[Bitboard; NrOf::SQUARES]; Sides::BOTH],
    pub rook: Vec<Bitboard>,
    pub bishop: Vec<Bitboard>,
    rook_magics: [Magic; NrOf::SQUARES],
    bishop_magics: [Magic; NrOf::SQUARES],
}

impl MoveGenerator {
    pub fn new() -> Self {
        let magics: Magic = Default::default();

        let mut mg = Self {
            king: [EMPTY; NrOf::SQUARES],
            knight: [EMPTY; NrOf::SQUARES],
            pawns: [[EMPTY; NrOf::SQUARES]; Sides::BOTH],
            rook: vec![EMPTY; ROOK_TABLE_SIZE],
            bishop: vec![EMPTY; BISHOP_TABLE_SIZE],
            rook_magics: [magics; NrOf::SQUARES],
            bishop_magics: [magics; NrOf::SQUARES],
        };

        mg.init_king_attack();
        mg.init_knight_attack();
        mg.init_pawn_attack();
        mg.init_magics(Pieces::ROOK);
        mg.init_magics(Pieces::BISHOP);
        mg
    }

    // Generates moves for the side that is to move. The MoveType parameter
    // determines if all moves, or only captures need to be generated.
    pub fn generate_moves(&self, board: &Board, move_list: &mut MoveList, move_type: MoveType) {
        self.piece(board, Pieces::KING, move_list, move_type);
        self.piece(board, Pieces::KNIGHT, move_list, move_type);
        self.piece(board, Pieces::ROOK, move_list, move_type);
        self.piece(board, Pieces::BISHOP, move_list, move_type);
        self.piece(board, Pieces::QUEEN, move_list, move_type);
        self.pawns(board, move_list, move_type);
        if move_type == MoveType::All || move_type == MoveType::Quiet {
            self.castling(board, move_list);
        }
    }

    pub fn get_non_slider_moves(&self, piece: Piece, square: Square) -> Bitboard {
        match piece {
            Pieces::KING => self.king[square],
            Pieces::KNIGHT => self.knight[square],
            _ => panic!("Not a king or knight {piece}"),
        }
    }

    pub fn get_slider_moves(&self, piece: Piece, square: Square, occupancy: Bitboard) -> Bitboard {
        match piece {
            Pieces::ROOK => {
                let index = self.rook_magics[square].get_index(occupancy);
                self.rook[index]
            }
            Pieces::BISHOP => {
                let index = self.bishop_magics[square].get_index(occupancy);
                self.bishop[index]
            }
            Pieces::QUEEN => {
                let r_index = self.rook_magics[square].get_index(occupancy);
                let b_index = self.bishop_magics[square].get_index(occupancy);
                self.rook[r_index] ^ self.bishop[b_index]
            }
            _ => panic!("Not a ROOK or BISHOP but {piece}"),
        }
    }

    pub fn get_pawn_attacks(&self, side: Side, square: Square) -> Bitboard {
        self.pawns[side][square]
    }
}

// GET ACTUAL PSEUDO LEGAL MOVES!!!!
impl MoveGenerator {
    pub fn piece(
        &self,
        board: &Board,
        piece: Piece,
        move_list: &mut MoveList,
        move_type: MoveType,
    ) {
        let player = board.side_to_move();
        let bb_occupancy = board.occupancy();
        // find all square that are empty occupied by our pieces and opponent pieces
        let bb_empty = !bb_occupancy;
        let bb_own_pieces = board.bb_side[player];
        let bb_opponent_pieces = board.bb_side[board.side_to_not_move()];
        let mut bb_pieces = board.get_pieces(piece, player);
        // Generate all moves for each piece
        while bb_pieces > 0 {
            let from = bits::next(&mut bb_pieces);
            let bb_target = match piece {
                Pieces::KING | Pieces::KNIGHT => self.get_non_slider_moves(piece, from),
                Pieces::ROOK | Pieces::BISHOP | Pieces::QUEEN => {
                    self.get_slider_moves(piece, from, bb_occupancy)
                }
                _ => panic!("Not a valid piece {piece}"),
            };

            // generate move according to movetype
            let bb_moves = match move_type {
                MoveType::All => bb_target & !bb_own_pieces,
                MoveType::Quiet => bb_target & bb_empty,
                MoveType::Capture => bb_target & bb_opponent_pieces,
            };

            self.add_move(board, piece, from, bb_moves, move_list);
        }
    }

    pub fn pawns(&self, board: &Board, list: &mut MoveList, mt: MoveType) {
        const UP: i8 = 8;
        const DOWN: i8 = -8;

        // Create shorthand variables.
        let us = board.side_to_move();
        let bb_opponent_pieces = board.bb_side[board.side_to_not_move()];
        let bb_empty = !board.occupancy();
        let bb_fourth = BB_RANKS[Board::fourth_rank(us)];
        let direction = if us == Sides::WHITE { UP } else { DOWN };
        let rotation_count = (NrOf::SQUARES as i8 + direction) as u32;
        let mut bb_pawns = board.get_pieces(Pieces::PAWN, us);

        // As long as there are pawns, generate moves for each of them.
        while bb_pawns > 0 {
            let from = bits::next(&mut bb_pawns);
            let to = (from as i8 + direction) as usize;
            let mut bb_moves = 0;

            // Generate pawn pushes
            if mt == MoveType::All || mt == MoveType::Quiet {
                let bb_push = BB_SQUARES[to];
                let bb_one_step = bb_push & bb_empty;
                let bb_two_step = bb_one_step.rotate_left(rotation_count) & bb_empty & bb_fourth;
                bb_moves |= bb_one_step | bb_two_step;
            }

            // Generate pawn captures
            if mt == MoveType::All || mt == MoveType::Capture {
                let bb_targets = self.get_pawn_attacks(us, from);
                let bb_captures = bb_targets & bb_opponent_pieces;
                let bb_ep_capture = match board.gamestate.en_passant {
                    Some(ep) => bb_targets & BB_SQUARES[ep as usize],
                    None => 0,
                };
                bb_moves |= bb_captures | bb_ep_capture;
            }

            self.add_move(board, Pieces::PAWN, from, bb_moves, list);
        }
    }

    pub fn castling(&self, board: &Board, move_list: &mut MoveList) -> bool {
        let player = board.side_to_move();
        let opponent = board.side_to_not_move();

        let castle_perm_w = (board.gamestate.castling & (Castling::WK | Castling::WQ)) > 0;
        let castle_perm_b = (board.gamestate.castling & (Castling::BK | Castling::BQ)) > 0;

        let bb_occupancy = board.occupancy();

        let mut bb_king = board.get_pieces(Pieces::KING, player);
        let from = bits::next(&mut bb_king);
        // White castling
        if castle_perm_w && player == Sides::WHITE {
            // kingside
            if board.gamestate.castling & Castling::WK > 0 {
                let bb_kingside_blockers = BB_SQUARES[Squares::F1] | BB_SQUARES[Squares::G1];
                let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

                if !is_kingside_blocked
                    && !self.square_attacked(board, opponent, Squares::E1)
                    && !self.square_attacked(board, opponent, Squares::F1)
                {
                    let to = BB_SQUARES[from] << 2;
                    self.add_move(board, Pieces::KING, from, to, move_list);
                }
            }

            if board.gamestate.castling & Castling::WQ > 0 {
                // Queenside
                let bb_queenside_blockers =
                    BB_SQUARES[Squares::B1] | BB_SQUARES[Squares::C1] | BB_SQUARES[Squares::D1];
                let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

                if !is_queenside_blocked
                    && !self.square_attacked(board, opponent, Squares::E1)
                    && !self.square_attacked(board, opponent, Squares::D1)
                {
                    let to = BB_SQUARES[from] >> 2;
                    self.add_move(board, Pieces::KING, from, to, move_list);
                }
            }
        }

        // Black castling
        if player == Sides::BLACK && castle_perm_b {
            // Kingside
            if board.gamestate.castling & Castling::BK > 0 {
                let bb_kingside_blockers = BB_SQUARES[Squares::F8] | BB_SQUARES[Squares::G8];
                let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

                if !is_kingside_blocked
                    && !self.square_attacked(board, opponent, Squares::E8)
                    && !self.square_attacked(board, opponent, Squares::F8)
                {
                    let to = BB_SQUARES[from] << 2;
                    self.add_move(board, Pieces::KING, from, to, move_list);
                }
            }

            // Queenside
            if board.gamestate.castling & Castling::BQ > 0 {
                let bb_queenside_blockers =
                    BB_SQUARES[Squares::B8] | BB_SQUARES[Squares::C8] | BB_SQUARES[Squares::D8];
                let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

                if !is_queenside_blocked
                    && !self.square_attacked(board, opponent, Squares::E8)
                    && !self.square_attacked(board, opponent, Squares::D8)
                {
                    let to = BB_SQUARES[from] >> 2;
                    self.add_move(board, Pieces::KING, from, to, move_list);
                }
            }
        }

        move_list.len() > 0
    }

    pub fn add_move(
        &self,
        board: &Board,
        piece: Piece,
        from: Square,
        to: Bitboard,
        move_list: &mut MoveList,
    ) {
        let mut bb_to = to;
        let player = board.side_to_move();
        let promotion_rank = Board::promotion_rank(player);
        let is_pawn = piece == Pieces::PAWN;

        while bb_to > 0 {
            let to_square = bits::next(&mut bb_to);
            let capture = board.piece_list[to_square];
            let en_passant = match board.gamestate.en_passant {
                Some(square) => is_pawn && (square as usize == to_square),
                None => false,
            };

            let promotion = is_pawn && Board::square_on_rank(to_square, promotion_rank);
            let double_push = is_pawn && ((to_square as i8 - from as i8).abs() == 16);
            let castling = (piece == Pieces::KING) && ((to_square as i8 - from as i8).abs() == 2);

            // add all data into a 64 bit variable
            let mut move_data = (piece)
                | from << Shift::FROM_SQ
                | to_square << Shift::TO_SQ
                | capture << Shift::CAPTURE
                | (en_passant as usize) << Shift::EN_PASSANT
                | (double_push as usize) << Shift::DOUBLE_STEP
                | (castling as usize) << Shift::CASTLING;

            // Simulate the move on a temporary board
            let mut cloned = board.clone();

            if !promotion {
                move_data |= Pieces::NONE << Shift::PROMOTION;
                if cloned.make_move(Move::new(move_data), &self) {
                    move_list.push(Move::new(move_data));
                }
            } else {
                // ... Or push 4 possible promotion pieces
                PROMOTION_PIECES.iter().for_each(|piece| {
                    let promotion_piece = *piece << Shift::PROMOTION;
                    if cloned.make_move(Move::new(move_data), &self) {
                        move_list.push(Move::new(move_data | promotion_piece));
                    }
                });
            }
            std::mem::drop(cloned);
        }
    }
}

impl MoveGenerator {
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    // Determine if a square is attacked by 'attacker', on the given board.
    pub fn square_attacked(&self, board: &Board, attacker: Side, square: Square) -> bool {
        let attackers = board.bb_pieces[attacker];

        // Use the super-piece method: get the moves for each piece,
        // starting from the given square. This provides the sqaures where
        // a piece has to be, to be able to reach the given square.
        let occupancy = board.occupancy();
        let bb_king = self.get_non_slider_moves(Pieces::KING, square);
        let bb_rook = self.get_slider_moves(Pieces::ROOK, square, occupancy);
        let bb_bishop = self.get_slider_moves(Pieces::BISHOP, square, occupancy);
        let bb_knight = self.get_non_slider_moves(Pieces::KNIGHT, square);
        let bb_pawns = self.get_pawn_attacks(attacker ^ 1, square);
        let bb_queen = bb_rook | bb_bishop;

        if (bb_king & attackers[Pieces::KING] > 0)
            || (bb_rook & attackers[Pieces::ROOK] > 0)
            || (bb_queen & attackers[Pieces::QUEEN] > 0)
            || (bb_bishop & attackers[Pieces::BISHOP] > 0)
            || (bb_knight & attackers[Pieces::KNIGHT] > 0)
            || (bb_pawns & attackers[Pieces::PAWN] > 0)
        {
            println!("bb:BLACK {:?}", print_bitboard(board.bb_side[Sides::BLACK]));
            println!("bb:WHITE {:?}", print_bitboard(board.bb_side[Sides::WHITE]));
        }

        // Then determine if such a piece is actually there: see if a rook
        // is on one of the squares a rook has to be to reach the given
        // square. Same for the queen, knight, etc... As soon as one is
        // found, the square is attacked.
        (bb_king & attackers[Pieces::KING] > 0)
            || (bb_rook & attackers[Pieces::ROOK] > 0)
            || (bb_queen & attackers[Pieces::QUEEN] > 0)
            || (bb_bishop & attackers[Pieces::BISHOP] > 0)
            || (bb_knight & attackers[Pieces::KNIGHT] > 0)
            || (bb_pawns & attackers[Pieces::PAWN] > 0)
    }
}

// ------------------ Test for movegeneration -------------------- \\

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::*;

    // Reference:
    // https://www.chessprogramming.org/Perft_Results
    #[test]
    fn depth_nodes() {
        let positions = vec![
            //"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // -> depth 5 OK
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", // -> not OK
                                                                                    //"8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", // -> dpeth 6 not OK
                                                                                    //"r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", //-> not Ok to much
                                                                                    //"rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                                                                                    // "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        ];

        for position in positions {
            let mut board = Board::new();
            let _ = board.read_fen(Some(position));

            let move_generator = MoveGenerator::new();
            let mut move_list = MoveList::new();

            let mut total_nodes: i32 = 0;

            let depth = 2;

            let mut perft_result: HashMap<String, i32> = Default::default();

            let elapsed_time = measure_time(|| {
                let _ = move_generator.generate_moves(&board, &mut move_list, MoveType::All);
                for mov in move_list.moves.iter() {
                    if mov.data > 0 {
                        board.make_move(*mov, &move_generator);
                        let the_move =
                            format!("{}{}", SQUARE_NAME[mov.from()], SQUARE_NAME[mov.to()]);
                        let nodes = perft_results(depth, &mut board, &move_generator, &the_move);
                        total_nodes += nodes;
                        perft_result.insert(the_move, nodes);
                        board.unmake();
                    }
                }
            });

            let mut sorted_vec: Vec<_> = perft_result.into_iter().collect();

            // Sort the vector based on the keys
            sorted_vec.sort_by(|a, b| a.0.cmp(&b.0));

            // Iterate over the sorted vector
            for (key, value) in sorted_vec {
                println!("{}: {}", key, value);
            }

            println!("\nPosition: {}\n", position);

            println!(
                "Depth: {} ply     Result: {} nodes    Time: {:?} milliseconds\n",
                depth + 1,
                total_nodes,
                elapsed_time.as_millis()
            );

            println!(
                "---------------------------------------------------------------------------\n"
            );
        }
    }

    // Test helper classes //
    fn measure_time<F: FnOnce()>(function: F) -> Duration {
        let start_time = Instant::now();
        function();
        let end_time = Instant::now();
        end_time - start_time
    }

    fn perft_results(
        depth: u8,
        board: &mut Board,
        move_generator: &MoveGenerator,
        current_move: &str,
    ) -> i32 {
        if depth == 0 {
            return 1;
        }

        let mut move_list = MoveList::new();
        let _ = move_generator.generate_moves(&board, &mut move_list, MoveType::All);

        let mut total_nodes = 0;

        for mov in move_list.moves.iter() {
            if mov.data > 0 {
                if board.make_move(*mov, move_generator) {
                    let nodes = perft_results(depth - 1, board, &move_generator, current_move);
                    total_nodes += nodes;
                    board.unmake();
                }
            }
        }

        total_nodes
    }
}

/* Move list contains al moves, however nodes per move aren't correct */
