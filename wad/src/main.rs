use std::{fs::File, path::PathBuf};

use clap::{Parser, Subcommand};
use wad::{parse_wad, rebuild_wad};

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Unpack {
        wad_file: PathBuf,
        output_dir: PathBuf,
        json_out: PathBuf,
    },
    Pack {
        json_in: PathBuf,
        output_file: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Unpack {
            wad_file,
            output_dir,
            json_out,
        } => {
            let manifest = parse_wad(wad_file, output_dir)?;

            let json_out = File::create(json_out)?;

            serde_json::to_writer_pretty(json_out, &manifest)?;
        }
        Command::Pack {
            json_in,
            output_file,
        } => {
            let json_in = File::open(json_in)?;

            let manifest = serde_json::from_reader(json_in)?;

            rebuild_wad(manifest, output_file)?;
        }
    }

    Ok(())
}
