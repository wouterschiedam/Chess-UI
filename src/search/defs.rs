use std::{sync::Arc, time::Instant};

use crate::{
    board::Board,
    defs::MAX_PLY,
    engine::defs::Information,
    movegen::{
        defs::{Move, ShortMove},
        MoveGenerator,
    },
};
use crossbeam_channel::{Receiver, Sender};
// Some const for searching
pub const INF: i16 = 25_000;
pub const CHECKMATE: i16 = 24_000;
pub const CHECKMATE_THRESHOLD: i16 = 23_900;
pub const STALEMATE: i16 = 0;
pub const DRAW: i16 = 0;
pub const CHECK_TERMINATION: usize = 0x7FF; // 2.047 nodes
pub const SEND_STATS: usize = 0x7FFFF; // 524.287 nodes
pub const MIN_TIME_STATS: u128 = 2_000; // Minimum time for sending stats
pub const MIN_TIME_CURR_MOVE: u128 = 1_000; // Minimum time for sending curr_move
pub const MAX_KILLER_MOVES: usize = 2;

pub type SearchResult = (Move, SearchTerminate);
type KillerMoves = [[ShortMove; MAX_KILLER_MOVES]; MAX_PLY as usize];

// Ways to terminate a search.
#[derive(PartialEq, Copy, Clone)]
pub enum SearchTerminate {
    Stop,    // Search is halted.
    Quit,    // Search module is quit completely.
    Nothing, // No command received yet.
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct GameTime {
    pub white_time: u128,
    pub black_time: u128,
    pub white_time_incr: u128,
    pub black_time_incr: u128,
    pub moves_to_go: Option<usize>,
}

impl GameTime {
    pub fn new(
        wtime: u128,
        btime: u128,
        white_time_incr: u128,
        black_time_incr: u128,
        moves_to_go: Option<usize>,
    ) -> Self {
        Self {
            white_time: wtime,
            black_time: btime,
            white_time_incr,
            black_time_incr,
            moves_to_go,
        }
    }
}

#[derive(PartialEq)]
// These commands can be used by the engine thread to control the search.
pub enum SearchControl {
    Start(SearchParams),
    Stop,
    Quit,
    Nothing,
}

// SearchMode lists how the search termination criteria will be evaluated,
// to see if the search has to be stopped.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SearchMode {
    Depth,    // Run until requested depth is reached.
    MoveTime, // Run until 'time per move' is used up.
    Nodes,    // Run until the number of requested nodes was reached.
    GameTime, // Search determines when to quit, depending on available time.
    Infinite, // Run forever, until the 'stop' command is received.
    Nothing,  // No search mode has been defined.
}

// This struct holds all the search parameters as set by the engine thread.
// (These parameters are either default, or provided by the user interface
// before the game starts.)
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct SearchParams {
    pub depth: i8,               // Maximum depth to search to
    pub move_time: u128,         // Maximum time per move to search
    pub nodes: usize,            // Maximum number of nodes to search
    pub game_time: GameTime,     // Time available for entire game
    pub search_mode: SearchMode, // Defines the mode to search in
    pub quiet: bool,             // No intermediate search stats updates
}

impl SearchParams {
    pub fn new() -> Self {
        Self {
            depth: MAX_PLY,
            move_time: 0,
            nodes: 0,
            game_time: GameTime::new(0, 0, 0, 0, None),
            search_mode: SearchMode::Nothing,
            quiet: false,
        }
    }

    pub fn is_game_time(&self) -> bool {
        self.search_mode == SearchMode::GameTime
    }
}

#[derive(PartialEq, Copy, Clone)]
pub struct SearchInfo {
    pub start_time: Option<Instant>,
    pub seldepth: i8, // Maximum selective depth reached
    pub depth: i8,
    pub nodes: usize,
    pub ply: i8,
    pub killer_moves: KillerMoves,
    pub terminated: SearchTerminate,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self {
            start_time: None,
            depth: 0,
            seldepth: 0,
            nodes: 0,
            ply: 0,
            killer_moves: [[ShortMove::new(0); MAX_KILLER_MOVES]; MAX_PLY as usize],
            terminated: SearchTerminate::Nothing,
        }
    }

    pub fn start_timer(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn time_elapsed(&self) -> u128 {
        if let Some(x) = self.start_time {
            x.elapsed().as_millis()
        } else {
            0
        }
    }

    pub fn interupted(&self) -> bool {
        self.terminated != SearchTerminate::Nothing
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
// This struct holds the currently searched move, and its move number in
// the list of legal moves. This struct is sent through the engine thread
// to Comm, to be transmitted to the (G)UI.
pub struct SearchCurrentMove {
    pub curr_move: Move,
    pub curr_move_number: u8,
}

impl SearchCurrentMove {
    pub fn new(curr_move: Move, curr_move_number: u8) -> Self {
        Self {
            curr_move,
            curr_move_number,
        }
    }
}

// This struct holds search statistics. These will be sent through the
// engine thread to Comm, to be transmitted to the (G)UI.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct SearchStats {
    pub time: u128,     // Time spent searching
    pub nodes: usize,   // Number of nodes searched
    pub nps: usize,     // Speed in nodes per second
    pub hash_full: u16, // TT full in permille
}

impl SearchStats {
    pub fn new(time: u128, nodes: usize, nps: usize, hash_full: u16) -> Self {
        Self {
            time,
            nodes,
            nps,
            hash_full,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SearchSummary {
    pub depth: i8,    // depth reached during search
    pub seldepth: i8, // Maximum selective depth reached
    pub time: u128,   // milliseconds
    pub cp: i16,      // centipawns score
    pub mate: u8,     // mate in X moves
    pub nodes: usize, // nodes searched
    pub nps: usize,   // nodes per second
    // pub hash_full: u16, // TT use in permille
    pub pv: Vec<Move>, // Principal Variation
}

impl SearchSummary {
    pub fn pv_as_string(&self) -> String {
        let mut pv = String::from("");
        for next_move in self.pv.iter() {
            let m = format!(" {}", next_move.as_string());
            pv.push_str(&m[..]);
        }
        pv.trim().to_string()
    }
}

pub struct SearchRefs<'a> {
    pub board: &'a mut Board,
    pub move_generator: &'a MoveGenerator,
    pub search_info: &'a mut SearchInfo,
    pub search_params: &'a mut SearchParams,
    pub control_rx: &'a Receiver<SearchControl>,
    pub report_tx: &'a Sender<Information>,
}

// This struct holds all the reports a search can send to the engine.
#[derive(PartialEq, Debug)]
pub enum SearchReport {
    Finished(Move),                       // Search done. Contains the best move.
    SearchSummary(SearchSummary),         // Periodic intermediate results.
    SearchCurrentMove(SearchCurrentMove), // Move currently searched.
    SearchStats(SearchStats),             // General search statistics
}
