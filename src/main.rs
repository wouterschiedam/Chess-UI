use ui::ui::run;
pub mod board;
pub mod defs;
pub mod evaluation;
pub mod extra;
pub mod movegen;
pub mod ui;

fn main() {
    let _ = run();
}
