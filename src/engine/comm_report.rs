use crate::{
    comm::{uci::UciReport, CommControl, CommReport},
    defs::FEN_START_POSITION,
    search::defs::{SearchControl, SearchMode, SearchParams, SearchReport},
};

use super::Engine;

impl Engine {
    pub fn comm_reports(&mut self, comm: &CommReport) {
        match comm {
            CommReport::Uci(c) => self.comm_report_uci(c),
        }
    }

    fn comm_report_uci(&mut self, ucireport: &UciReport) {
        let mut sp = SearchParams::new();
        sp.quiet = self.settings.quiet;
        match ucireport {
            UciReport::Uci => {
                self.comm.send(CommControl::Identify);
            }

            UciReport::UciNewGame => {
                let _ = self
                    .board
                    .lock()
                    .expect("error locking board")
                    .read_fen(Some(FEN_START_POSITION));
            }
            UciReport::IsReady => {
                self.comm.send(CommControl::Ready);
            }

            // UciReport::SetOption(option) => (),
            UciReport::Position(fen, moves) => {
                let fen_result = self
                    .board
                    .lock()
                    .expect("Error locking board")
                    .read_fen(Some(fen));

                if fen_result.is_ok() {
                    for m in moves.iter() {
                        let ok = self.execute_move(m.clone());
                        if !ok {
                            let msg = format!("{}: {}", m, "illigal move");
                            self.comm.send(CommControl::InfoString(msg));
                            break;
                        }
                    }
                }
            }
            UciReport::GoInfinite => {
                sp.search_mode = SearchMode::Infinite;
                self.search.send(SearchControl::Start(sp));
            }
            UciReport::GoDepth(depth) => {
                sp.depth = *depth;
                sp.search_mode = SearchMode::Depth;
                self.search.send(SearchControl::Start(sp));
            }
            UciReport::GoMoveTime(t) => {
                sp.move_time = *t;
                sp.search_mode = SearchMode::MoveTime;
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::GoNodes(nodes) => {
                sp.nodes = *nodes;
                sp.search_mode = SearchMode::Nodes;
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::GoGameTime(gt) => {
                sp.game_time = *gt;
                sp.search_mode = SearchMode::GameTime;
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::Quit => self.quit(),
            UciReport::Stop => self.search.send(SearchControl::Stop),
            UciReport::Board => self.comm.send(CommControl::PrintBoard),
            UciReport::Unknown => (),
        }
    }
}
