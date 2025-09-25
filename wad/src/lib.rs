use std::{
    fs::{File, create_dir_all},
    io::{Read, Seek, copy},
    path::PathBuf,
};

use anyhow::anyhow;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
pub struct WADFile {
    pub offset: u32,
    pub length: u32,
}

impl WADFile {
    pub fn end(&self) -> bool {
        self.offset == 0 && self.length == 0
    }
}

#[derive(Debug)]
pub struct WADHeader {
    pub files: [WADFile; 256],
}

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub timestamp: DateTime<Local>,
    pub files: Vec<PathBuf>,
}

pub fn parse_wad(wad_file: PathBuf, output_dir: PathBuf) -> anyhow::Result<Manifest> {
    if !output_dir.is_dir() {
        return Err(anyhow!("Output dir is not a dir"));
    }

    let mut file = File::open(&wad_file)?;
    let mut header = WADHeader {
        files: [WADFile {
            offset: 0,
            length: 0,
        }; 256],
    };

    let mut offset = [0u8; 4];
    let mut length = [0u8; 4];
    for i in 0..256 {
        file.read_exact(&mut offset)?;
        file.read_exact(&mut length)?;

        header.files[i] = WADFile {
            offset: u32::from_le_bytes(offset),
            length: u32::from_le_bytes(length),
        };
    }

    create_dir_all(&output_dir)?;

    let mut manifest = Manifest {
        timestamp: Local::now(),
        files: Vec::new(),
    };

    let mut file = File::open(&wad_file)?;

    // skip header
    file.seek(std::io::SeekFrom::Start(2048))?;

    for (i, wfile) in header.files.iter().enumerate() {
        if wfile.end() {
            break;
        }

        let mut dst = output_dir.clone();
        dst.push(format!("{}.bin", i));

        let mut dst_file = File::create(&dst)?;

        let mut remaining = wfile.length;

        while remaining > 0 {
            let to_copy = remaining.min(1024);
            let mut limited = file.by_ref().take(to_copy as u64);

            let written = copy(&mut limited, &mut dst_file)?;

            // EOF
            if written == 0 {
                break;
            }

            remaining -= written as u32;
        }

        manifest.files.push(dst);
    }

    Ok(manifest)
}
