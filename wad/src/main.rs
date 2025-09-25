use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    wad_file: PathBuf,
    output: PathBuf,
}

fn main() {
    let args = Args::parse();

    dbg!(args);
}
