use crate::defs::{Bitboard, Square};

// Get the next set bit form the bitboard and unset it. If a piece bitboard is given,
// this provides the location of the next piece of that type
pub fn next(bitboard: &mut Bitboard) -> Square {
    let square = bitboard.trailing_zeros() as Square;
    *bitboard ^= 1u64 << square;
    square
}
