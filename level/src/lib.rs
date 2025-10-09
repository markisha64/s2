use std::{
    fs::{File, create_dir_all},
    io::{Read, Seek, Write, copy},
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
    pub tex_0: PathBuf,
    pub tex_1: PathBuf,
    pub reverb: PathBuf,
    pub audio_buffers: Vec<PathBuf>,
    pub collision_data: PathBuf,
    pub model: PathBuf,
    pub something: Vec<PathBuf>,
    pub some_offsets: Vec<u32>,
    pub model_indices: Vec<u16>,
}

fn copy_limited(file: &mut File, dst_file: &mut File, remaining: u64) -> anyhow::Result<()> {
    let mut limited = Read::by_ref(file).take(remaining);

    copy(&mut limited, dst_file)?;

    Ok(())
}

fn write_16bpp_tim_header(
    dst_file: &mut File,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
) -> anyhow::Result<()> {
    // TIM header for 16bpp, no CLUT
    let magic: u32 = 0x10; // TIM file identifier
    let flags: u32 = 0x02; // 16bpp, no CLUT

    // Each pixel = 2 bytes, so data size = width * height * 2
    // + 12 for the pixel block header (len + x + y + w + h)
    let pixel_data_size: u32 = (w as u32 * h as u32 * 2) + 12;

    // Write TIM magic and flags
    dst_file.write_all(&magic.to_le_bytes())?;
    dst_file.write_all(&flags.to_le_bytes())?;

    // Write pixel block size
    dst_file.write_all(&pixel_data_size.to_le_bytes())?;

    // X, Y, Width, Height (all little-endian 16-bit)
    dst_file.write_all(&x.to_le_bytes())?; // X
    dst_file.write_all(&y.to_le_bytes())?; // Y
    dst_file.write_all(&w.to_le_bytes())?; // Width in pixels
    dst_file.write_all(&h.to_le_bytes())?; // Height in pixels

    Ok(())
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

    file.seek(std::io::SeekFrom::Start(header.tex_and_audio.offset as u64))?;

    let mut tex_0 = output_dir.clone();
    tex_0.push("tex_0.tim");

    let mut dst_file = File::create(&tex_0)?;

    write_16bpp_tim_header(&mut dst_file, 512, 0, 512, 256)?;

    // tex part
    copy_limited(&mut file, &mut dst_file, 256 * 1024)?;

    let mut tex_1 = output_dir.clone();
    tex_1.push("tex_1.tim");

    let mut dst_file = File::create(&tex_1)?;

    write_16bpp_tim_header(&mut dst_file, 512, 256, 512, 256)?;

    // tex part
    copy_limited(&mut file, &mut dst_file, 256 * 1024)?;

    file.seek(std::io::SeekFrom::Start(
        header.tex_and_audio.offset as u64 + 512 * 1024,
    ))?;

    let mut reverb = output_dir.clone();
    reverb.push("a_reverb.bin");

    let mut dst_file = File::create(&reverb)?;

    // reverb part
    copy_limited(&mut file, &mut dst_file, 24 * 1024)?;

    let mut a_buffers = Vec::new();

    for i in 0..8 {
        let lb = i * 64 * 1024;
        let len = (header.tex_and_audio.length - (lb + 512 * 1024 + 24 * 1024)).min(64 * 1024);

        file.seek(std::io::SeekFrom::Start(
            header.tex_and_audio.offset as u64 + 512 * 1024 + 24 * 1024 + lb as u64,
        ))?;

        if len == 0 {
            break;
        }

        let mut buf = output_dir.clone();
        buf.push(format!("a_buf_{i}.vag"));

        let mut dst_file = File::create(&buf)?;

        let mut vag_header = [0u8; 48];

        vag_header[0..4].copy_from_slice("VAGp".as_bytes());
        vag_header[4..8].copy_from_slice(&(0x20u32.to_be_bytes()));
        vag_header[12..16].copy_from_slice(&(len.min(64 * 1024).to_be_bytes()));
        vag_header[16..20].copy_from_slice(&(11025u32.to_be_bytes()));

        dst_file.write(&vag_header)?;

        // reverb part
        copy_limited(&mut file, &mut dst_file, len.min(64 * 1024) as u64)?;

        a_buffers.push(buf);

        if len < 64 * 1024 {
            break;
        }
    }

    let mut make_file = |wfile: WADFile, name: &str| -> anyhow::Result<PathBuf> {
        file.seek(std::io::SeekFrom::Start(wfile.offset as u64))?;

        let mut dst = output_dir.clone();
        dst.push(name);

        let mut dst_file = File::create(&dst)?;

        copy_limited(&mut file, &mut dst_file, wfile.length as u64)?;

        Ok(dst)
    };

    let collision_data = make_file(header.collision_data, "collision_data.bin")?;
    let model = make_file(header.model, "model.bin")?;

    let something = (0..5)
        .map(|i| make_file(header.something[i], format!("s_{i}.bin").as_str()))
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(LevelManifest {
        timestamp: Local::now(),
        tex_0,
        tex_1,
        reverb,
        audio_buffers: a_buffers,
        collision_data,
        model,
        something,
        some_offsets: header.some_offsets.try_into()?,
        model_indices: header.model_indices.try_into()?,
    })
}
