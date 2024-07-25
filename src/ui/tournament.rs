use std::fs::File;
use std::io::{self, Write};

#[derive(Debug)]
pub struct Tournament {
    games_to_play: usize,
    pub games_played: usize,
    engine1_wins: usize,
    engine2_wins: usize,
    draws: usize,
    log_file: File,
}

impl Tournament {
    pub fn new(games_to_play: usize, log_file_path: &str) -> io::Result<Self> {
        let log_file = File::create(log_file_path)?;
        Ok(Self {
            games_to_play,
            games_played: 0,
            engine1_wins: 0,
            engine2_wins: 0,
            draws: 0,
            log_file,
        })
    }

    pub fn log_result(&mut self, result: &str) -> io::Result<()> {
        self.games_played += 1;
        writeln!(self.log_file, "Game {}: {}", self.games_played, result)?;
        match result {
            "Engine 1 wins" => self.engine1_wins += 1,
            "Engine 2 wins" => self.engine2_wins += 1,
            "Draw" => self.draws += 1,
            _ => (),
        }
        Ok(())
    }

    pub fn tournament_over(&self) -> bool {
        self.games_played >= self.games_to_play
    }
}
