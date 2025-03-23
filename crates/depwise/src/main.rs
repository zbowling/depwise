use clap::Parser;
use depwise::cli::{Cli, execute};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    execute(args)
}
