use std::{
    fs::{File, create_dir_all},
    io::{Read, Seek, copy},
    path::PathBuf,
};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use wad::WADFile;

#[derive(Debug)]
pub struct LevelHeader {
    pub tex_and_audio: WADFile,
    pub collision_data: WADFile,
    pub model: WADFile,
    pub something: [WADFile; 5],
    pub some_offsets: [u32; 64],
    pub model_indices: [u16; 64],
}

#[derive(Serialize, Deserialize)]
pub struct LevelManifest {
    pub timestamp: DateTime<Local>,
    pub tex_and_audio: PathBuf,
    pub collision_data: PathBuf,
    pub model: PathBuf,
    pub something: Vec<PathBuf>,
    pub some_offsets: Vec<u32>,
    pub model_indices: Vec<u16>,
}

pub fn parse_level(level_file: PathBuf, output_dir: PathBuf) -> anyhow::Result<LevelManifest> {
    let mut file = File::open(&level_file)?;
    let mut header = LevelHeader {
        tex_and_audio: WADFile {
            offset: 0,
            length: 0,
        },
        collision_data: WADFile {
            offset: 0,
            length: 0,
        },
        model: WADFile {
            offset: 0,
            length: 0,
        },
        something: [WADFile {
            offset: 0,
            length: 0,
        }; 5],
        some_offsets: [0; 64],
        model_indices: [0; 64],
    };

    let mut offset = [0u8; 4];
    let mut length = [0u8; 4];

    file.read_exact(&mut offset)?;
    file.read_exact(&mut length)?;

    header.tex_and_audio = WADFile {
        offset: u32::from_le_bytes(offset),
        length: u32::from_le_bytes(length),
    };

    file.read_exact(&mut offset)?;
    file.read_exact(&mut length)?;

    header.collision_data = WADFile {
        offset: u32::from_le_bytes(offset),
        length: u32::from_le_bytes(length),
    };

    file.read_exact(&mut offset)?;
    file.read_exact(&mut length)?;

    header.model = WADFile {
        offset: u32::from_le_bytes(offset),
        length: u32::from_le_bytes(length),
    };

    for i in 0..5 {
        file.read_exact(&mut offset)?;
        file.read_exact(&mut length)?;

        header.something[i] = WADFile {
            offset: u32::from_le_bytes(offset),
            length: u32::from_le_bytes(length),
        };
    }

    for i in 0..64 {
        file.read_exact(&mut offset)?;

        header.some_offsets[i] = u32::from_le_bytes(offset);
    }

    let mut short_buffer = [0u8; 2];
    for i in 0..64 {
        file.read_exact(&mut short_buffer)?;

        header.model_indices[i] = u16::from_le_bytes(short_buffer);
    }

    create_dir_all(&output_dir)?;

    let mut file = File::open(&level_file)?;

    let mut make_file = |wfile: WADFile, name: &str| -> anyhow::Result<PathBuf> {
        file.seek(std::io::SeekFrom::Start(wfile.offset as u64))?;

        let mut dst = output_dir.clone();
        dst.push(name);

        let mut dst_file = File::create(&dst)?;

        let mut remaining = wfile.length;

        while remaining > 0 {
            let to_copy = remaining.min(1024);
            let mut limited = Read::by_ref(&mut file).take(to_copy as u64);

            let written = copy(&mut limited, &mut dst_file)?;

            // EOF
            if written == 0 {
                break;
            }

            remaining -= written as u32;
        }

        Ok(dst)
    };

    let tex_and_audio = make_file(header.tex_and_audio, "tex_and_audio.bin")?;
    let collision_data = make_file(header.collision_data, "collision_data.bin")?;
    let model = make_file(header.model, "model.bin")?;

    let something = (0..5)
        .map(|i| make_file(header.something[i], format!("s_{i}.bin").as_str()))
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(LevelManifest {
        timestamp: Local::now(),
        tex_and_audio,
        collision_data,
        model,
        something,
        some_offsets: header.some_offsets.try_into()?,
        model_indices: header.model_indices.try_into()?,
    })
}
