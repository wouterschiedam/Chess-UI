use super::defs::Move;
use crate::defs::MAXLEGAL_MOVES;
use std::mem;

#[derive(Copy, Clone, Debug)]
pub struct MoveList {
    pub moves: [Move; MAXLEGAL_MOVES as usize],
    count: u8,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move { data: 0 }; MAXLEGAL_MOVES as usize],
            count: 0,
        }
    }

    // Used to store a move in the move list.
    pub fn push(&mut self, m: Move) {
        self.moves[self.count as usize] = m;
        self.count += 1;
    }

    // Returns the number of moves in the move list.
    pub fn len(&self) -> u8 {
        self.count
    }

    // Return the move at the given index. If out of bounds, the program crashes.
    pub fn get_move(&self, index: u8) -> Move {
        self.moves[index as usize]
    }

    pub fn get_mut_move(&mut self, index: u8) -> &mut Move {
        &mut self.moves[index as usize]
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        unsafe {
            // Take two raw pointers to the moves.
            let ptr_a: *mut Move = &mut self.moves[a];
            let ptr_b: *mut Move = &mut self.moves[b];

            // Swap those pointers, so now the moves are swapped.
            std::ptr::swap(ptr_a, ptr_b);
        }
    }
}
