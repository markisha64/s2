use std::{
    fs::{File, create_dir_all},
    io::{Read, copy},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

// section 0
pub struct CollisionHeader {
    pub section_1_offset: u32,
    pub section_0_data_len: u32,
}

pub struct CollisionSection1 {
    pub section_3_offset: u32,
    pub section_2_offset: u32,
}

pub struct CollisionSection2 {
    pub section_2_data_len: u32,
    pub section_4_offset: u32,
}

pub struct CollisionSection3 {
    pub section_5_offset: u32,
}

pub struct CollisionSection5 {
    pub collision_types_offset: u32,
}

pub struct CollisionTypes {
    pub section_7_offset: u32,
    pub collision_types_len: u32,
}

pub struct CollisionSection7 {
    pub section_8_offset: u32,
}

#[derive(Debug)]
pub struct CollisionSection8 {
    pub section_9_offset: u32,
    pub triangle_count: u32,
    pub idfk_offset: u32,
    pub unk_0: u32,
    pub unk_1_offset: u32,
    pub unk_2_offset: u32,
    pub triangles_offset: u32,
    pub collision_flags_offset: u32,
    pub unk_3_offset: u32,
    pub unk_4_offset: u32,
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

pub fn parse_collision(
    collision_file: PathBuf,
    output_dir: PathBuf,
) -> anyhow::Result<CollisionManifest> {
    let mut file = File::open(&collision_file)?;
    create_dir_all(&output_dir)?;

    let mut buffer_0 = [0u8; 4];
    let mut buffer_1 = [0u8; 4];

    file.read_exact(&mut buffer_0)?;
    file.read_exact(&mut buffer_1)?;

    let header = CollisionHeader {
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

    let section_1 = CollisionSection1 {
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

    let section_2 = CollisionSection2 {
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

    let section_2_data_len = section_1.section_3_offset
        - 8
        - section_1_data_len * 28
        - 8
        - section_2.section_2_data_len * 4;

    let mut section_2_pb = output_dir.clone();
    section_2_pb.push("section_2.dat");

    let mut section_2_file = File::create(section_2_pb)?;

    copy_file(&mut file, &mut section_2_file, section_2_data_len as u64)?;

    file.read_exact(&mut buffer_0)?;

    let section_3 = CollisionSection3 {
        section_5_offset: u32::from_le_bytes(buffer_0),
    };

    let mut section_3_pb = output_dir.clone();
    section_3_pb.push("section_3.dat");

    let mut section_3_file = File::create(section_3_pb)?;

    copy_file(
        &mut file,
        &mut section_3_file,
        section_3.section_5_offset as u64 - 4,
    )?;

    file.read_exact(&mut buffer_0)?;

    let section_5 = CollisionSection5 {
        collision_types_offset: u32::from_le_bytes(buffer_0),
    };

    let mut section_5_pb = output_dir.clone();
    section_5_pb.push("section_5.dat");

    let mut section_5_file = File::create(section_5_pb)?;

    copy_file(
        &mut file,
        &mut section_5_file,
        section_5.collision_types_offset as u64 - 4,
    )?;

    file.read_exact(&mut buffer_0)?;
    file.read_exact(&mut buffer_1)?;

    let collision_types = CollisionTypes {
        section_7_offset: u32::from_le_bytes(buffer_0),
        collision_types_len: u32::from_le_bytes(buffer_1),
    };

    let mut collision_types_pb = output_dir.clone();
    collision_types_pb.push("collision_types.dat");

    let mut collision_types_file = File::create(collision_types_pb)?;

    copy_file(
        &mut file,
        &mut collision_types_file,
        collision_types.section_7_offset as u64 - 8,
    )?;

    file.read_exact(&mut buffer_0)?;

    let section_7 = CollisionSection7 {
        section_8_offset: u32::from_le_bytes(buffer_0),
    };

    let mut section_7_pb = output_dir.clone();
    section_7_pb.push("section_7.dat");

    let mut section_7_file = File::create(section_7_pb)?;

    copy_file(
        &mut file,
        &mut section_7_file,
        section_7.section_8_offset as u64 - 4,
    )?;

    let mut vec_3_pb = output_dir.clone();
    vec_3_pb.push("vec_3.dat");

    let mut vec_3_file = File::create(vec_3_pb)?;

    copy_file(&mut file, &mut vec_3_file, 12)?;

    file.read_exact(&mut buffer_0)?;
    file.read_exact(&mut buffer_1)?;

    let section_9_offset = u32::from_le_bytes(buffer_0);
    let triangle_count = u32::from_le_bytes(buffer_1);

    file.read_exact(&mut buffer_0)?;
    file.read_exact(&mut buffer_1)?;

    let idfk_offset = u32::from_le_bytes(buffer_0);
    let unk_0 = u32::from_le_bytes(buffer_1);

    file.read_exact(&mut buffer_0)?;
    file.read_exact(&mut buffer_1)?;

    let unk_1_offset = u32::from_le_bytes(buffer_0);
    let unk_2_offset = u32::from_le_bytes(buffer_1);

    file.read_exact(&mut buffer_0)?;
    file.read_exact(&mut buffer_1)?;

    let triangles_offset = u32::from_le_bytes(buffer_0);
    let collision_flags_offset = u32::from_le_bytes(buffer_1);

    file.read_exact(&mut buffer_0)?;
    file.read_exact(&mut buffer_1)?;

    let section_8 = CollisionSection8 {
        section_9_offset,
        triangle_count,
        idfk_offset,
        unk_0,
        unk_1_offset,
        unk_2_offset,
        triangles_offset,
        collision_flags_offset,
        unk_3_offset: u32::from_le_bytes(buffer_0),
        unk_4_offset: u32::from_le_bytes(buffer_1),
    };

    let mut files = [
        ("section_8_idfk.bin", idfk_offset),
        ("section_8_unk_0.bin", unk_0),
        ("section_8_unk_1.bin", unk_1_offset),
        ("section_8_unk_2.bin", unk_2_offset),
        ("section_8_triangles.bin", triangles_offset),
        ("section_8_collision_flags.bin", collision_flags_offset),
        ("section_8_unk_3.bin", section_8.unk_3_offset),
        ("section_8_unk_4.bin", section_8.unk_4_offset),
        ("", section_9_offset),
    ];
    files.sort_by(|(_, a), (_, b)| a.cmp(b));

    for x in files.windows(2) {
        let f1 = x[0];
        let f2 = x[1];

        if f1.1 == 0 {
            continue;
        }

        let mut ff_pb = output_dir.clone();
        ff_pb.push(f1.0);

        let mut ff_file = File::create(ff_pb)?;

        copy_file(&mut file, &mut ff_file, (f2.1 - f1.1) as u64)?;
    }

    let mut tail = output_dir.clone();
    tail.push("tail.bin");

    let mut tail_file = File::create(tail)?;

    // copy to end
    copy_file(&mut file, &mut tail_file, 999999999)?;

    Ok(CollisionManifest {})
}
