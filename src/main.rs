use std::{
    fs::{self, File},
    path::PathBuf,
    process::Command,
};

use clap::{Parser, Subcommand};
use wad::{parse_wad, rebuild_wad};

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Unpack and prepare
    Unpack {
        /// Target game bin file
        target_bin: PathBuf,
    },
    /// Rebuild iso
    Repack {
        /// Output folder
        name: String,
    },
}

const LEVELS: [&str; 58] = [
    "level_10_summer_forest_code.ovl",
    "level_10_summer_forest_data.wad",
    "level_11_glimmer_code.ovl",
    "level_11_glimmer_data.wad",
    "level_12_idol_springs_code.ovl",
    "level_12_idol_springs_data.wad",
    "level_13_colossus_code.ovl",
    "level_13_colossus_data.wad",
    "level_21_hurricos_code.ovl",
    "level_21_hurricos_data.wad",
    "level_22_aquaria_towers_code.ovl",
    "level_22_aquaria_towers_data.wad",
    "level_23_sunny_beach_code.ovl",
    "level_23_sunny_beach_data.wad",
    "level_25_ocean_speedway_code.ovl",
    "level_25_ocean_speedway_data.wad",
    "level_26_crushs_dungeon_code.ovl",
    "level_26_crushs_dungeon_data.wad",
    "level_30_autumn_plains_code.ovl",
    "level_30_autumn_plains_data.wad",
    "level_31_skelos_badlands_code.ovl",
    "level_31_skelos_badlands_data.wad",
    "level_32_crystal_glacier_code.ovl",
    "level_32_crystal_glacier_data.wad",
    "level_33_breeze_harbor_code.ovl",
    "level_33_breeze_harbor_data.wad",
    "level_34_zephyr_code.ovl",
    "level_34_zephyr_data.wad",
    "level_35_metro_speedway_code.ovl",
    "level_35_metro_speedway_data.wad",
    "level_41_scorch_code.ovl",
    "level_41_scorch_data.wad",
    "level_42_shady_oasis_code.ovl",
    "level_42_shady_oasis_data.wad",
    "level_43_magma_cone_code.ovl",
    "level_43_magma_cone_data.wad",
    "level_44_fracture_hills_code.ovl",
    "level_44_fracture_hills_data.wad",
    "level_45_icy_speedway_code.ovl",
    "level_45_icy_speedway_data.wad",
    "level_46_gulps_overlook_code.ovl",
    "level_46_gulps_overlook_data.wad",
    "level_50_winter_tundra_code.ovl",
    "level_50_winter_tundra_data.wad",
    "level_51_mystic_marsh_code.ovl",
    "level_51_mystic_marsh_data.wad",
    "level_52_cloud_temples_code.ovl",
    "level_52_cloud_temples_data.wad",
    "level_55_canyon_speedway_code.ovl",
    "level_55_canyon_speedway_data.wad",
    "level_61_robotica_farms_code.ovl",
    "level_61_robotica_farms_data.wad",
    "level_62_metropolis_code.ovl",
    "level_62_metropolis_data.wad",
    "level_65_dragon_shores_code.ovl",
    "level_65_dragon_shores_data.wad",
    "level_66_riptos_arena_code.ovl",
    "level_66_riptos_arena_data.wad",
];

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        SubCommand::Unpack { target_bin } => {
            println!("Unpacking ISO");
            Command::new("dumpsxiso")
                .args(["-x", "extract", "-s", "out.xml"])
                .arg(target_bin)
                .output()?;

            println!("Unpacking main WAD file");
            let wad_file = ["extract", "WAD.WAD"].iter().collect();
            let output_dir = ["extract", "WAD"].iter().collect();
            let mut manifest = parse_wad(wad_file, output_dir)?;

            println!("Moving files");
            let pb: PathBuf = ["extract", "WAD", "levels"].iter().collect();
            fs::create_dir_all(pb)?;

            for (i, wfile) in manifest.files.iter_mut().skip(14).take(29 * 2).enumerate() {
                let mut new_name = wfile.clone();
                new_name.pop();
                new_name.push("levels");
                new_name.push(LEVELS[i]);

                fs::rename(&wfile, &new_name)?;

                *wfile = new_name;

                // level extract logic (unsure if ill even use)
                // if LEVELS[i].ends_with(".wad") {
                //     let mut output_dir = wfile.clone();
                //     output_dir.pop();
                //     output_dir.push(LEVELS[i].trim_end_matches(".wad"));

                //     let level_manifest = parse_wad(wfile.clone(), output_dir.clone())?;

                //     let mut level_json = output_dir.clone();
                //     level_json.push("level.json");

                //     let level_json_f = File::create(level_json)?;

                //     serde_json::to_writer_pretty(level_json_f, &level_manifest)?;
                // }
            }

            println!("Save WAD.WAD.json");
            let was_wad_json =
                File::create(["extract", "WAD.WAD.json"].iter().collect::<PathBuf>())?;

            serde_json::to_writer_pretty(was_wad_json, &manifest)?;
        }
        SubCommand::Repack { name } => {
            println!("Rebuild WAD.WAD");
            let manifest_pbuf: PathBuf = ["extract", "WAD.WAD.json"].iter().collect();

            let manifest_file = File::open(manifest_pbuf)?;

            let manifest = serde_json::from_reader(manifest_file)?;

            let wad_wad = ["extract", "WAD.WAD"].iter().collect();
            rebuild_wad(manifest, wad_wad)?;

            println!("Rebuild ISO");
            // make sure folder exists
            fs::create_dir_all("out")?;

            let out_b: PathBuf = ["out", format!("{}.bin", name).as_str()].iter().collect();
            let out_c: PathBuf = ["out", format!("{}.cue", name).as_str()].iter().collect();

            Command::new("mkpsxiso")
                .arg("-o")
                .arg(out_b)
                .arg("-c")
                .arg(out_c)
                .arg("out.xml")
                .output()?;
        }
    }

    Ok(())
}
