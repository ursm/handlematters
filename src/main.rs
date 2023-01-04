use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser)]
#[command(version, author, about)]
struct Opts {
    /// Input file [default: stdin]
    #[arg()]
    file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let mut reader: Box<dyn Read> = match opts.file {
        Some(path) => {
            let path_str = path.to_string_lossy();

            if path_str == "-" {
                Box::new(io::stdin())
            } else {
                Box::new(File::open(path.clone()).with_context(|| format!("failed to open file: {path_str}"))?)
            }
        }
        None => Box::new(io::stdin()),
    };

    handlematters::run(&mut reader)?;

    Ok(())
}
