pub mod about;
pub mod comm_report;
pub mod defs;
pub mod search_report;
pub mod utils;
use std::sync::{Arc, Mutex};

use crate::{
    board::{defs::Pieces, Board},
    comm::{uci::Uci, CommControl, IComm},
    defs::{EngineRunResult, Piece, Sides},
    extra::cmdline::Cmdline,
    movegen::{defs::print_bitboard, MoveGenerator},
    search::{
        defs::{SearchControl, SearchInfo},
        Search,
    },
};
use crossbeam_channel::Receiver;

use self::defs::{EngineOptionDefaults, Information, Settings};

pub struct Engine {
    quit: bool,
    cmdline: Cmdline, // Command line interpreter.
    settings: Settings,
    board: Arc<Mutex<Board>>,
    pub comm: Box<dyn IComm>, // Communications (active).
    movegen: Arc<MoveGenerator>,
    search: Search,
    pub info_receiver: Option<Receiver<Information>>, // Receiver for incoming information.
}

impl Engine {
    pub fn new() -> Self {
        // Determine if the compiled engine is 32 or 64-bit
        let is_64_bit = std::mem::size_of::<usize>() == 8;

        let cmdline = Cmdline::new();
        // Get engine settings from the command-line.
        let threads = cmdline.threads();
        let quiet = cmdline.has_quiet();
        let tt_size = cmdline.hash();

        let comm = Box::new(Uci::new());

        let tt_max = if is_64_bit {
            EngineOptionDefaults::HASH_MAX_64_BIT
        } else {
            EngineOptionDefaults::HASH_MAX_32_BIT
        };

        Self {
            quit: false,
            settings: Settings {
                threads,
                quiet,
                tt_size,
            },
            comm,
            cmdline,
            board: Arc::new(Mutex::new(Board::new())),
            movegen: Arc::new(MoveGenerator::new()),
            search: Search::new(),
            info_receiver: None,
        }
    }

    pub fn run(&mut self) -> EngineRunResult {
        self.print_about(&self.settings);
        println!();

        self.setup_position()?;

        // engine runs in the main loop where it checks for legal moves.
        self.main_loop();

        Ok(())
    }

    // This function quits Commm, Search, and then the engine thread itself.
    pub fn quit(&mut self) {
        self.search.send(SearchControl::Quit);
        self.comm.send(CommControl::Quit);
        self.quit = true;
    }
}
