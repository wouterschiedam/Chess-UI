use crate::{comm::CommControl, search::defs::SearchReport};

use super::Engine;

impl Engine {
    pub fn search_report(&mut self, search: &SearchReport) {
        match search {
            SearchReport::Finished(m) => {
                self.comm.send(CommControl::BestMove(*m));
                self.comm.send(CommControl::Update);
            }
            SearchReport::SearchCurrentMove(cm) => self.comm.send(CommControl::SearchCurrMove(*cm)),
            SearchReport::SearchStats(ss) => self.comm.send(CommControl::SearchStats(*ss)),
            SearchReport::SearchSummary(sm) => {
                self.comm.send(CommControl::SearchSummary(sm.clone()))
            }
        }
    }
}
