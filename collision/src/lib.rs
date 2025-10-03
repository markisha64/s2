use std::{
    fs::{File, create_dir_all},
    io::{Read, copy},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

pub struct CollisionHeader {
    pub offset: u32,
    pub unk_len: u32,
}

#[derive(Serialize, Deserialize)]
pub struct CollisionManifest {}

fn copy_file(file: &mut File, dst_file: &mut File, mut remaining: u64) -> anyhow::Result<()> {
    while remaining > 0 {
        let to_copy = remaining.min(1024);
        let mut limited = Read::by_ref(file).take(to_copy);

        let written = copy(&mut limited, dst_file)?;

        // EOF
        if written == 0 {
            break;
        }

        remaining -= written;
    }

    Ok(())
}

pub fn parse_colission(
    colission_file: PathBuf,
    output_dir: PathBuf,
) -> anyhow::Result<CollisionManifest> {
    let mut file = File::open(&colission_file)?;
    create_dir_all(&output_dir)?;

    let mut buffer_0 = [0u8; 4];
    let mut buffer_1 = [0u8; 4];
    file.read_exact(&mut buffer_0)?;
    file.read_exact(&mut buffer_1)?;

    let offset = u32::from_le_bytes(buffer_0);
    let unk_len = u32::from_le_bytes(buffer_1);

    let mut file_0 = output_dir.clone();
    file_0.push("file_0.bin");

    let mut dst_file = File::create(&file_0)?;

    copy_file(&mut file, &mut dst_file, offset as u64 - 8)?;

    let mut file_1 = output_dir.clone();
    file_1.push("file_1.bin");

    let mut dst_file = File::create(&file_1)?;

    copy_file(&mut file, &mut dst_file, 999999)?;

    Ok(CollisionManifest {})
}
