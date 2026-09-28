use crate::cli::CliRunner;

mod cli;
mod constants;
mod display;
mod emulator;
mod processor;
mod sound;

fn main() {
    let runner = CliRunner::new();
    runner.run();
}
