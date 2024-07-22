use crate::defs::About;

use super::{defs::Settings, Engine};

impl Engine {
    // Print information about the engine.
    pub fn print_about(&self, s: &Settings) {
        let bits = std::mem::size_of::<usize>() * 8;
        let hash = if s.tt_size == 0 {
            String::from("off")
        } else {
            format!("{} MB", s.tt_size)
        };
        let threads = if s.threads == 1 {
            String::from("4")
        } else {
            format!("{} (unused, always 1)", s.threads)
        };

        // println!("{:<10} {} {}", "Engine:", About::ENGINE, About::VERSION);
        // println!("{:<10} {}", "Author:", About::AUTHOR);
        // println!("{:<10} {}", "EMail:", About::EMAIL);
        // println!("{:<10} {bits}-bit", "Type:");
        // println!("{:<10} {hash}", "Hash:");
        // println!("{:<10} {threads}", "Threads:");
    }
}
