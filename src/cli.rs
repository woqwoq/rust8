use std::path::PathBuf;

use clap::Parser;

use crate::emulator::{Emulator, EmulatorError};

#[derive(Parser)]
#[command(about = "A portable CHIP8 Emulator written in Rust.")]
pub struct Cli {
    /// Path to ROM file
    #[arg(short, long, value_name = "FILE")]
    pub file: PathBuf,
}

pub struct CliRunner {
    cli: Cli,
}

impl CliRunner {
    pub fn new() -> Self {
        CliRunner { cli: Cli::parse() }
    }

    fn handle_error(e: EmulatorError) {
        eprintln!("Fatal error encountered: {e}");
    }

    pub fn run(&self) {
        match Emulator::from_rom_file(self.cli.file.clone()) {
            Err(e) => Self::handle_error(e),
            Ok(mut e) => {
                if let Err(e) = e.run() {
                    Self::handle_error(e)
                }
            }
        }
    }
}
