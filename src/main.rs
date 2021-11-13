use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser)]
#[clap(version, author, about)]
struct Opts {
    /// Input file [default: stdin]
    #[clap(parse(from_os_str))]
    file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let mut reader: Box<dyn Read> = match opts.file {
        Some(path) => {
            let file = File::open(path.clone()).with_context(|| format!("failed to open file: {}", path.clone().to_string_lossy()))?;

            Box::new(file)
        }
        None => Box::new(io::stdin()),
    };

    let mut src = String::new();

    reader.read_to_string(&mut src)?;

    let output = handlematters::render(&src, |stderr| eprint!("{}", stderr))?;

    print!("{}", output);

    Ok(())
}
