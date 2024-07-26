use self::{
    defs::{Pieces, Ranks, BB_SQUARES, PIECE_CHAR_CAPS, SQUARE_BITBOARD},
    gamestate::GameState,
    history::History,
    zobrist::ZobristKey,
    zobrist::ZobristRandoms,
};
use crate::{
    board::defs::SQUARE_NAME,
    defs::{Bitboard, NrOf, Piece, Side, Sides, Square, EMPTY},
    evaluation::{
        defs::PIECE_VALUES,
        material,
        psqt::{self, FLIP, PSQT_MG},
    },
    extra::{bits, parse::algebraic_square_to_number},
    movegen::{
        defs::{print_bitboard, Move, Shift},
        PROMOTION_PIECES,
    },
};
use std::sync::Arc;

pub mod defs;
mod fen;
mod gamestate;
mod history;
mod makemove;
mod utils;
mod zobrist;
#[derive(Clone, Debug)]
pub struct Board {
    pub bb_pieces: [[Bitboard; NrOf::PIECE_TYPES]; Sides::BOTH],
    pub bb_side: [Bitboard; Sides::BOTH],
    pub piece_list: [Piece; NrOf::SQUARES],
    pub gamestate: GameState,
    pub zr: Arc<ZobristRandoms>,
    pub history: History,
}

impl Board {
    // Create a new board with start pos or a fen string
    pub fn new() -> Self {
        Self {
            bb_pieces: [[EMPTY; NrOf::PIECE_TYPES]; Sides::BOTH],
            bb_side: [EMPTY; Sides::BOTH],
            piece_list: [Pieces::NONE; NrOf::SQUARES],
            gamestate: GameState::new(),
            zr: Arc::new(ZobristRandoms::new()),
            history: History::new(),
        }
    }

    pub fn build() -> Self {
        let mut board = Board::new();
        let _ = board.read_fen(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ));

        board
    }

    pub fn get_pieces(&self, piece: Piece, side: Side) -> Bitboard {
        self.bb_pieces[side][piece]
    }

    pub fn occupancy(&self) -> Bitboard {
        self.bb_side[Sides::WHITE] | self.bb_side[Sides::BLACK]
    }

    pub fn side_to_move(&self) -> usize {
        self.gamestate.active_color as usize
    }

    pub fn side_to_not_move(&self) -> usize {
        (self.gamestate.active_color ^ 1) as usize
    }

    pub fn king_square(&self, side: Side) -> Square {
        self.bb_pieces[side][Pieces::KING].trailing_zeros() as Square
    }

    // Remove piece for given [Side][Square][Piece]
    pub fn remove_piece(&mut self, side: Side, piece: Piece, square: Square) {
        // remove from pieces / side and piecelist
        self.bb_pieces[side][piece] ^= BB_SQUARES[square];
        self.bb_side[side] ^= BB_SQUARES[square];
        self.piece_list[square] = Pieces::NONE;
        self.gamestate.zobrist_key ^= self.zr.piece(side, square, piece);

        // Update material
        if !self.gamestate.material[side] < PIECE_VALUES[piece] {
            self.gamestate.material[side] -= PIECE_VALUES[piece];
        } else {
            self.gamestate.material[side] = 0;
        }

        let flip = side == Sides::WHITE;
        let s = if flip { FLIP[square] } else { square };
        self.gamestate.psqt[side] -= PSQT_MG[piece][s] as i16;
    }

    pub fn put_piece(&mut self, side: Side, piece: Piece, square: Square) {
        // set piece / side and piecelist
        self.bb_pieces[side][piece] |= BB_SQUARES[square];
        self.bb_side[side] |= BB_SQUARES[square];
        self.piece_list[square] = piece;
        self.gamestate.zobrist_key ^= self.zr.piece(side, square, piece);

        // update material
        self.gamestate.material[side] += PIECE_VALUES[piece];

        let flip = side == Sides::WHITE;
        let s = if flip { FLIP[square] } else { square };
        self.gamestate.psqt[side] += PSQT_MG[piece][s] as i16;
    }

    pub fn move_piece(&mut self, side: Side, piece: Piece, from: Square, to: Square) {
        // remove from square and add to square
        self.remove_piece(side, piece, from);
        self.put_piece(side, piece, to);
    }

    pub fn set_ep_square(&mut self, square: Square) {
        self.gamestate.zobrist_key ^= self.zr.en_passant(self.gamestate.en_passant);
        self.gamestate.en_passant = Some(square as u8);
        self.gamestate.zobrist_key ^= self.zr.en_passant(self.gamestate.en_passant);
    }

    pub fn clear_ep_square(&mut self) {
        self.gamestate.zobrist_key ^= self.zr.en_passant(self.gamestate.en_passant);
        self.gamestate.en_passant = None;
        self.gamestate.zobrist_key ^= self.zr.en_passant(self.gamestate.en_passant);
    }

    pub fn update_castling_perm(&mut self, new_perm: u8) {
        self.gamestate.zobrist_key ^= self.zr.castling(self.gamestate.castling);
        self.gamestate.castling = new_perm;
        self.gamestate.zobrist_key ^= self.zr.castling(self.gamestate.castling);
    }

    pub fn swap_side(&mut self) {
        self.gamestate.zobrist_key ^= self.zr.side(self.gamestate.active_color as usize);
        self.gamestate.active_color ^= 1;
        self.gamestate.zobrist_key ^= self.zr.side(self.gamestate.active_color as usize);
    }

    // Helper funcitons for GUI
    pub fn color_on(&self, position: Option<Square>) -> usize {
        let square_number = position;
        let bb_white = self.bb_side[Sides::WHITE];
        let bb_black = self.bb_side[Sides::BLACK];

        let is_white = bb_white & (1 << square_number.unwrap()) != 0;
        let is_black = bb_black & (1 << square_number.unwrap()) != 0;
        if is_white {
            return Sides::WHITE;
        }
        if is_black {
            return Sides::BLACK;
        }

        Sides::BOTH
    }

    pub fn piece_on(&self, position: Option<Square>) -> Option<usize> {
        let square_number = position;
        let white_pieces = self.bb_side[Sides::WHITE];
        let black_pieces = self.bb_side[Sides::BLACK];

        let is_white = white_pieces & (1 << &square_number.unwrap()) != 0;
        let is_black = black_pieces & (1 << &square_number.unwrap()) != 0;

        if is_white {
            let bb_w = self.bb_pieces[Sides::WHITE];
            for (piece, w) in bb_w.iter().enumerate() {
                if w & (1 << &square_number.unwrap()) != 0 {
                    return Some(piece);
                }
            }
        }
        if is_black {
            let bb_b = self.bb_pieces[Sides::BLACK];
            for (piece, b) in bb_b.iter().enumerate() {
                if b & (1 << &square_number.unwrap()) != 0 {
                    return Some(piece);
                }
            }
        }

        None
    }

    pub fn get_square(&self, position: (usize, usize)) -> Option<Square> {
        let row = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        let square_number =
            algebraic_square_to_number(format!("{}{}", row[position.0 - 1], position.1).as_str());

        square_number
    }

    pub fn generate_move_data(
        &self,
        from: &usize,
        to: &Option<usize>,
        side: bool,
        promotion_piece: Option<usize>,
    ) -> usize {
        let piece = self.piece_on(Some(*from)).unwrap();
        let is_pawn = piece == Pieces::PAWN;
        let to_square = to.unwrap();
        let capture = self.piece_list[to_square];
        let en_passant = match self.gamestate.en_passant {
            Some(square) => is_pawn && (square as usize == to_square),
            None => false,
        };

        let promotion_rank = if side { Ranks::R8 } else { Ranks::R1 };
        let promotion = is_pawn && Board::square_on_rank(to_square, promotion_rank);
        let double_push = is_pawn && ((to_square as i8 - *from as i8).abs() == 16);
        let castling = (piece == Pieces::KING) && ((to_square as i8 - *from as i8).abs() == 2);

        let mut move_data = (piece)
            | from << Shift::FROM_SQ
            | to_square << Shift::TO_SQ
            | capture << Shift::CAPTURE
            | (en_passant as usize) << Shift::EN_PASSANT
            | (double_push as usize) << Shift::DOUBLE_STEP
            | (castling as usize) << Shift::CASTLING;

        if !promotion {
            move_data |= Pieces::NONE << Shift::PROMOTION;
        } else {
            if let Some(promotion_piece) = promotion_piece {
                move_data |= promotion_piece << Shift::PROMOTION;
            } else {
                // ... Or push 4 possible promotion pieces
                for &promotion_piece in PROMOTION_PIECES.iter() {
                    move_data = move_data | (promotion_piece << Shift::PROMOTION);
                }
            }
        }

        move_data
    }

    // End helper functions
}

// Private board functions (for startup)
impl Board {
    fn reset(&mut self) {
        self.bb_pieces = [[0; NrOf::PIECE_TYPES]; Sides::BOTH];
        self.bb_side = [EMPTY; Sides::BOTH];
        self.piece_list = [Pieces::NONE; NrOf::SQUARES];
        self.gamestate = GameState::new()
    }

    // Initialize board
    pub fn init(&mut self) {
        // Get bitboard for both white and black based on starting pos
        let pieces_per_side = self.init_pieces_per_side_bb();
        self.bb_side[Sides::WHITE] = pieces_per_side.0;
        self.bb_side[Sides::BLACK] = pieces_per_side.1;

        // Init piecelist, zobrist_key and material count
        self.piece_list = self.init_piece_list();
        self.gamestate.zobrist_key = self.init_zobrist_key();

        let material = material::count(self);
        self.gamestate.material[Sides::WHITE] = material.0;
        self.gamestate.material[Sides::BLACK] = material.1;

        let psqt = psqt::apply(self);
        self.gamestate.psqt[Sides::WHITE] = psqt.0;
        self.gamestate.psqt[Sides::BLACK] = psqt.1;
    }

    fn init_zobrist_key(&self) -> ZobristKey {
        let mut key: u64 = 0;

        let bb_w = self.bb_pieces[Sides::WHITE];
        let bb_b = self.bb_pieces[Sides::BLACK];

        for (piece_type, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            // Assume the first iteration; piece_type will be 0 (KING). The
            // following two statements will thus get all the pieces of
            // type "KING" for white and black. (This will obviously only
            // be one king, but with rooks, there will be two in the
            // starting position.)
            let mut white_pieces = *w;
            let mut black_pieces = *b;

            while white_pieces > 0 {
                let square = bits::next(&mut white_pieces);
                key ^= self.zr.piece(Sides::WHITE, square, piece_type);
            }

            while black_pieces > 0 {
                let square = bits::next(&mut black_pieces);
                key ^= self.zr.piece(Sides::BLACK, square, piece_type);
            }
        }

        key ^= self.zr.castling(self.gamestate.castling);
        key ^= self.zr.side(self.gamestate.active_color as usize);
        key ^= self.zr.en_passant(self.gamestate.en_passant);

        key
    }
    fn init_piece_list(&self) -> [Piece; NrOf::SQUARES] {
        let bb_w = self.bb_pieces[Sides::WHITE];
        let bb_b = self.bb_pieces[Sides::BLACK];

        let mut piece_list: [Piece; NrOf::SQUARES] = [Pieces::NONE; NrOf::SQUARES];

        for (piece_type, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            let mut white_pieces = *w;
            let mut black_pieces = *b;

            // add white piece to piece list
            while white_pieces > 0 {
                let square = bits::next(&mut white_pieces);
                piece_list[square] = piece_type;
            }

            // add black piece to piece list
            while black_pieces > 0 {
                let square = bits::next(&mut black_pieces);
                piece_list[square] = piece_type;
            }
        }

        piece_list
    }

    // init pieces per side bitboard
    fn init_pieces_per_side_bb(&self) -> (Bitboard, Bitboard) {
        let mut bb_white: Bitboard = 0;
        let mut bb_black: Bitboard = 0;

        for (bb_w, bb_b) in self.bb_pieces[Sides::WHITE]
            .iter()
            .zip(self.bb_pieces[Sides::BLACK].iter())
        {
            bb_white |= *bb_w;
            bb_black |= *bb_b;
        }

        (bb_white, bb_black)
    }
}
