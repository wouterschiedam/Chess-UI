use crate::movegen::MoveGenerator;
use crate::{
    board::defs::{Pieces, RangeOf, PIECE_NAME, SQUARE_NAME},
    defs::{Bitboard, Piece, Square, EMPTY},
    movegen::{defs::Magic, BISHOP_TABLE_SIZE, ROOK_TABLE_SIZE},
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
pub fn find_magics(piece: Piece) {
    // First check if we're actually dealing with a rook or a bishop.
    let ok = piece == Pieces::ROOK || piece == Pieces::BISHOP;
    assert!(ok, "Illegal piece: {piece}");

    // Create working variables.
    let is_rook = piece == Pieces::ROOK;
    let mut rook_table: Vec<Bitboard> = vec![EMPTY; ROOK_TABLE_SIZE];
    let mut bishop_table: Vec<Bitboard> = vec![EMPTY; BISHOP_TABLE_SIZE];
    let mut random = ChaChaRng::from_entropy();
    let mut offset = 0;

    println!("Finding magics for: {}", PIECE_NAME[piece]);
    for sq in RangeOf::SQUARES {
        // Create the mask for either the rook or bishop.
        let r_mask = MoveGenerator::rook_mask(sq);
        let b_mask = MoveGenerator::bishop_mask(sq);
        let mask = if is_rook { r_mask } else { b_mask };

        // Precalculate needed values.
        let bits = mask.count_ones(); // Number of set bits in the mask
        let permutations = 2u64.pow(bits); // Number of blocker boards to be indexed.
        let end = offset + permutations - 1; // End index in the attack table.

        // Create blocker boards for the current mask.
        let blocker_boards = MoveGenerator::blocker_boards(mask);

        // Create attack boards for the current square/blocker combo (either
        // rook or bishop).
        let r_ab = MoveGenerator::rook_attack(sq, &blocker_boards);
        let b_ab = MoveGenerator::bishop_attack(sq, &blocker_boards);
        let attack_boards = if is_rook { r_ab } else { b_ab };

        // Done calculating needed data. Create a new magic.
        let mut try_this: Magic = Default::default(); // New magic
        let mut found = false; // While loop breaker if magic works;
        let mut attempts = 0; // Track needed attempts to find the magic.

        // Set up the new magic with the values we already know.
        try_this.mask = mask;
        try_this.shift = (64 - bits) as u8;
        try_this.offset = offset;
        // start looking for magic numbers that work
        while !found {
            attempts += 1;
            found = true;

            try_this.nr = random.gen::<u64>() & random.gen::<u64>() & random.gen::<u64>();

            // try every permutation for blocker boards on current square
            for i in 0..permutations {
                let next = i as usize;
                let index = try_this.get_index(blocker_boards[next]);

                // Use either rook or bishop table
                let r_table = &mut rook_table[..];
                let b_table = &mut bishop_table[..];
                let table: &mut [Bitboard] = if is_rook { r_table } else { b_table };

                // if table at index is EMPTY
                if table[index] == EMPTY {
                    // in range?
                    let fail_low = index < offset as usize;
                    let fail_high = index > end as usize;
                    assert!(!fail_low && !fail_high, "Indexing error.");

                    table[index] = attack_boards[next];
                } else {
                    // table at index is not empty. magic doesn't work. wipe part of table we are
                    // working with and try another number
                    for wipe_index in offset..=end {
                        table[wipe_index as usize] = EMPTY;
                    }
                    found = false;
                    break;
                }
            }
        }

        // We got out of the loop and found a random magic number that can
        // index all the attack boards for a rook/bishop for a single
        // square without a collision. Report this number.
        found_magic(sq, try_this, offset, end, attempts);

        // Set table offset for next magic.
        offset += permutations;
    }

    // Test if table is correct.
    let r_ts = ROOK_TABLE_SIZE as u64;
    let b_ts = BISHOP_TABLE_SIZE as u64;
    let expected = if is_rook { r_ts } else { b_ts };
    const ERROR: &str = "Creating magics failed. Permutations were skipped.";

    assert!(offset == expected, "{}", ERROR);
}

// Print the magic number.
fn found_magic(square: Square, m: Magic, offset: u64, end: u64, attempts: u64) {
    println!(
        "{}: {:24}u64 (offset: {:6}, end: {:6}, attempts: {})",
        SQUARE_NAME[square], m.nr, offset, end, attempts
    );
}
