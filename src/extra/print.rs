use colored::*;
use std::sync::{Arc, Mutex};

use crate::{
    board::{
        defs::{Pieces, RangeOf, SQUARE_NAME},
        Board,
    },
    defs::{Bitboard, NrOf, Piece, Sides, Square},
    evaluation::material::count,
    extra::print,
    movegen::defs::{castling_as_string, print_bitboard},
};
type AsciiBoard = [char; NrOf::SQUARES];

const CHAR_ES: char = '.';
const CHAR_WK: char = 'K';
const CHAR_WQ: char = 'Q';
const CHAR_WR: char = 'R';
const CHAR_WB: char = 'B';
const CHAR_WN: char = 'N';
const CHAR_WP: char = 'I';
const CHAR_BK: char = 'k';
const CHAR_BQ: char = 'q';
const CHAR_BR: char = 'r';
const CHAR_BB: char = 'b';
const CHAR_BN: char = 'n';
const CHAR_BP: char = 'i';

pub fn print_position(board: &Board, mark: Option<u8>) {
    let mut ascii_board: AsciiBoard = [CHAR_ES; NrOf::SQUARES];

    board_to_ascii(board, &mut ascii_board);
    ascii_to_console(&ascii_board, mark);
    game_metadata(board);
}

fn board_to_ascii(board: &Board, ascii_board: &mut AsciiBoard) {
    let bb_w = board.bb_pieces[Sides::WHITE];
    let bb_b = board.bb_pieces[Sides::BLACK];

    for (piece, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
        match piece {
            Pieces::KING => {
                put_character_on_square(*w, ascii_board, CHAR_WK);
                put_character_on_square(*b, ascii_board, CHAR_BK);
            }
            Pieces::QUEEN => {
                put_character_on_square(*w, ascii_board, CHAR_WQ);
                put_character_on_square(*b, ascii_board, CHAR_BQ);
            }
            Pieces::ROOK => {
                put_character_on_square(*w, ascii_board, CHAR_WR);
                put_character_on_square(*b, ascii_board, CHAR_BR);
            }
            Pieces::BISHOP => {
                put_character_on_square(*w, ascii_board, CHAR_WB);
                put_character_on_square(*b, ascii_board, CHAR_BB);
            }
            Pieces::KNIGHT => {
                put_character_on_square(*w, ascii_board, CHAR_WN);
                put_character_on_square(*b, ascii_board, CHAR_BN);
            }
            Pieces::PAWN => {
                put_character_on_square(*w, ascii_board, CHAR_WP);
                put_character_on_square(*b, ascii_board, CHAR_BP);
            }
            _ => (),
        }
    }
}

fn put_character_on_square(bitboard: Bitboard, ascii_board: &mut AsciiBoard, character: char) {
    for (i, square) in ascii_board.iter_mut().enumerate() {
        if (bitboard >> i) & 1 == 1 {
            *square = character;
        }
    }
}

fn ascii_to_console(ascii_board: &AsciiBoard, mark: Option<u8>) {
    let coordinate_alpha: &str = "ABCDEFGH";
    let mut coordinate_digit = NrOf::FILES;

    println!();

    for current_rank in RangeOf::RANKS.rev() {
        print!("{coordinate_digit}    ");
        for current_file in RangeOf::FILES {
            let square = (current_rank as usize * NrOf::FILES) + current_file as usize;
            let mut char = ascii_board[square];

            if char == '.' {
                char = ' ';
            }

            let cell = format!("{} ", char);

            let cell = if char.is_uppercase() {
                cell.white()
            } else {
                cell.black()
            };

            let cell = if (current_rank + current_file) % 2 != 0 {
                cell.on_truecolor(158, 93, 30)
            } else {
                cell.on_truecolor(205, 170, 125)
            };

            print!("{}", cell);
        }

        println!();
        println!();
        coordinate_digit -= 1;
    }

    print!("     ");
    for c in coordinate_alpha.chars() {
        print!("{c} ");
    }
    println!();
    println!();
}

fn game_metadata(board: &Board) {
    let is_white = (board.gamestate.active_color as usize) == Sides::WHITE;
    let turn = if is_white { "White" } else { "Black" };
    let castling = castling_as_string(board.gamestate.castling);

    let en_passant = match board.gamestate.en_passant {
        Some(ep) => SQUARE_NAME[ep as usize],
        _ => "-",
    };

    let half_moveclock = board.gamestate.halfclock_move;
    let full_movenumber = board.gamestate.fullmove_number;

    let eval = count(&board);
    println!("{:<20}{}", "White eval:", eval.0);
    println!("{:<20}{}", "Black eval:", eval.1);

    println!("{:<20}{:x}", "Zobrist key:", board.gamestate.zobrist_key);
    println!("{:<20}{}", "Active Color:", turn);
    println!("{:<20}{}", "Castling:", castling);
    println!("{:<20}{}", "En Passant:", en_passant);
    println!("{:<20}{}", "Half-move clock:", half_moveclock);
    println!("{:<20}{}", "Full-move number:", full_movenumber);
    println!();
}
