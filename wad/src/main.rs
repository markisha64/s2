use std::path::PathBuf;

use clap::Parser;
use wad::parse_wad;

#[derive(Parser, Debug)]
struct Args {
    wad_file: PathBuf,
    output: PathBuf,
}

fn main() {
    let args = Args::parse();

    parse_wad(args.wad_file, args.output).unwrap();
}
