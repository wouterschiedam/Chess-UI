use super::{
    defs::{Pieces, Squares, BB_SQUARES},
    Board,
};
use crate::{
    board::defs::SQUARE_NAME,
    defs::{Castling, NrOf, Piece, Side, Sides, Square},
    movegen::{
        defs::{print_bitboard, Move},
        MoveGenerator,
    },
};

// Castling Permissions Per Square
type CPSquare = [u8; NrOf::SQUARES];
const CASTLING_PERMS: CPSquare = castling_permissions_per_square();
const fn castling_permissions_per_square() -> CPSquare {
    // First set all squares grant all castling permissions. This means
    // moving a piece on such square doesn't have any effect on castling
    // permissions.
    let mut cp: CPSquare = [Castling::ALL; NrOf::SQUARES];

    // Now disable castling permissions when moving pieces on certain
    // squares. For example, when the piece (rook) on A1 moves, disable
    // white castling to the queenside.
    cp[Squares::A1] &= !Castling::WQ;
    cp[Squares::E1] &= !Castling::WK & !Castling::WQ;
    cp[Squares::H1] &= !Castling::WK;
    cp[Squares::A8] &= !Castling::BQ;
    cp[Squares::E8] &= !Castling::BK & !Castling::BQ;
    cp[Squares::H8] &= !Castling::BK;

    cp
}

impl Board {
    pub fn make_move(&mut self, m: Move, mg: &MoveGenerator) -> bool {
        let mut current_game_state = self.gamestate;
        current_game_state.next_move = m;
        self.history.push(current_game_state);

        // get player and opponent
        let player = self.side_to_move();
        let opponent = player ^ 1;

        // get all data we need
        let piece = m.piece();
        let from = m.from();
        let to = m.to();
        let captured = m.captured();
        let promoted = m.promoted();
        let castling = m.castling();
        let double_push = m.double_push();
        let en_passant = m.en_passant();

        let is_promotion = promoted != Pieces::NONE;
        let is_capture = captured != Pieces::NONE;
        let castling_perm = self.gamestate.castling > 0;
        // Base form not a pawn moves
        self.gamestate.halfclock_move += 1;

        // every move execpt double_push unsets ep square
        if self.gamestate.en_passant.is_some() {
            self.clear_ep_square();
        }

        // if move is a capture then remove it also reset halfmoveclock
        if is_capture {
            self.remove_piece(opponent, captured, to);
            self.gamestate.halfclock_move = 0;

            if captured == Pieces::ROOK && castling_perm {
                self.update_castling_perm(self.gamestate.castling & CASTLING_PERMS[to]);
            }
        }

        // Make move if NOT a pawn
        if piece != Pieces::PAWN {
            self.move_piece(player, piece, from, to);
        } else {
            // it is a pawn move also check for promotion and reset halfclock_move
            self.remove_piece(player, piece, from);
            self.put_piece(player, if !is_promotion { piece } else { promoted }, to);
            self.gamestate.halfclock_move = 0;

            // if en_passant remove opponent piece
            if en_passant {
                self.remove_piece(opponent, Pieces::PAWN, to ^ 8);
            }

            if double_push {
                self.set_ep_square(to ^ 8);
            }
        }

        // check if king / rook moves from start square if so remove perm
        if (piece == Pieces::KING || piece == Pieces::ROOK) & castling_perm {
            self.update_castling_perm(self.gamestate.castling & CASTLING_PERMS[from]);
        }

        // if castling move rook aswell
        if castling {
            match to {
                Squares::G1 => self.move_piece(player, Pieces::ROOK, Squares::H1, Squares::F1),
                Squares::C1 => self.move_piece(player, Pieces::ROOK, Squares::A1, Squares::D1),
                Squares::G8 => self.move_piece(player, Pieces::ROOK, Squares::H8, Squares::F8),
                Squares::C8 => self.move_piece(player, Pieces::ROOK, Squares::H8, Squares::F8),
                _ => panic!("Eror moving rook"),
            }
        }

        // swap player
        self.swap_side();

        // If black moved increase fullmove
        if player == Sides::BLACK {
            self.gamestate.fullmove_number += 1;
        }

        // VALID IF MOVE IS LEGAL
        let is_legal = !mg.square_attacked(self, opponent, self.king_square(player));
        if !is_legal {
            self.unmake();
        }

        is_legal
    }
}

// Unmake() reverses the last move. The game state is restored by popping it
// from the history array, all variables at once.
impl Board {
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub fn unmake(&mut self) {
        self.gamestate = self.history.pop();

        // Set "us" and "opponent"
        let player = self.side_to_move();
        let opponent = player ^ 1;

        // Dissect the move to undo
        let m = self.gamestate.next_move;
        let piece = m.piece();
        let from = m.from();
        let to = m.to();
        let captured = m.captured();
        let promoted = m.promoted();
        let castling = m.castling();
        let en_passant = m.en_passant();

        // Moving backwards...
        if promoted == Pieces::NONE {
            reverse_move(self, player, piece, to, from);
        } else {
            remove_piece(self, player, promoted, to);
            put_piece(self, player, Pieces::PAWN, from);
        }

        // The king's move was already undone as a normal move.
        // Now undo the correct castling rook move.
        if castling {
            match to {
                Squares::G1 => reverse_move(self, player, Pieces::ROOK, Squares::F1, Squares::H1),
                Squares::C1 => reverse_move(self, player, Pieces::ROOK, Squares::D1, Squares::A1),
                Squares::G8 => reverse_move(self, player, Pieces::ROOK, Squares::F8, Squares::H8),
                Squares::C8 => reverse_move(self, player, Pieces::ROOK, Squares::D8, Squares::A8),
                _ => panic!("Error: Reversing castling rook move."),
            };
        }

        // If a piece was captured, put it back onto the to-square
        if captured != Pieces::NONE {
            put_piece(self, opponent, captured, to);
        }

        // If this was an e-passant move, put the opponent's pawn back
        if en_passant {
            put_piece(self, opponent, Pieces::PAWN, to ^ 8);
        }
    }
}

// Removes a piece from the board without Zobrist key updates.
fn remove_piece(board: &mut Board, side: Side, piece: Piece, square: Square) {
    board.bb_pieces[side][piece] ^= BB_SQUARES[square];
    board.bb_side[side] ^= BB_SQUARES[square];
    board.piece_list[square] = Pieces::NONE;
}

// Puts a piece onto the board without Zobrist key updates.
fn put_piece(board: &mut Board, side: Side, piece: Piece, square: Square) {
    board.bb_pieces[side][piece] |= BB_SQUARES[square];
    board.bb_side[side] |= BB_SQUARES[square];
    board.piece_list[square] = piece;
}

// Moves a piece from one square to another.
fn reverse_move(board: &mut Board, side: Side, piece: Piece, remove: Square, put: Square) {
    remove_piece(board, side, piece, remove);
    put_piece(board, side, piece, put);
}
