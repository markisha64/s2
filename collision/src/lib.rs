use std::{
    fs::{File, create_dir_all},
    io::{Read, copy},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

// section 0
pub struct ColissionHeader {
    pub section_1_offset: u32,
    pub section_0_data_len: u32,
}

pub struct ColissionSection1 {
    pub section_3_offset: u32,
    pub section_2_offset: u32,
}

pub struct ColissionSection2 {
    pub section_2_data_len: u32,
    pub section_4_offset: u32,
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

    let header = ColissionHeader {
        section_1_offset: u32::from_le_bytes(buffer_0),
        section_0_data_len: u32::from_le_bytes(buffer_1),
    };

    let mut section_0_pb = output_dir.clone();
    section_0_pb.push("section_0.dat");

    let mut section_0_file = File::create(section_0_pb)?;

    copy_file(
        &mut file,
        &mut section_0_file,
        header.section_1_offset as u64 - 8,
    )?;

    file.read_exact(&mut buffer_0)?;
    file.read_exact(&mut buffer_1)?;

    let section_1 = ColissionSection1 {
        section_3_offset: u32::from_le_bytes(buffer_0),
        section_2_offset: u32::from_le_bytes(buffer_1),
    };

    let section_1_data_len = (section_1.section_2_offset - 4) / 28;

    let mut section_1_pb = output_dir.clone();
    section_1_pb.push("section_1.dat");

    let mut section_1_file = File::create(section_1_pb)?;

    copy_file(
        &mut file,
        &mut section_1_file,
        section_1_data_len as u64 * 28,
    )?;

    file.read_exact(&mut buffer_0)?;
    file.read_exact(&mut buffer_1)?;

    let section_2 = ColissionSection2 {
        section_2_data_len: u32::from_le_bytes(buffer_0),
        section_4_offset: u32::from_le_bytes(buffer_1),
    };

    let mut section_2_offsets_pb = output_dir.clone();
    section_2_offsets_pb.push("section_2_offsets.dat");

    let mut section_2_offsets_file = File::create(section_2_offsets_pb)?;

    copy_file(
        &mut file,
        &mut section_2_offsets_file,
        section_2.section_2_data_len as u64 * 4,
    )?;

    let mut tail = output_dir.clone();
    tail.push("tail.bin");

    let mut tail_file = File::create(tail)?;

    // copy to end
    copy_file(&mut file, &mut tail_file, 999999999)?;

    Ok(CollisionManifest {})
}
