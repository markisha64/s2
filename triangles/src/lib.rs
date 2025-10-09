use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

pub struct Vec3<T>(T, T, T);

pub struct Triangle {
    pub v1: Vec3<i32>,
    pub v2: Vec3<i32>,
    pub v3: Vec3<i32>,
}

pub fn read_tri(file: &mut File) -> anyhow::Result<Triangle> {
    let mut buffer_0 = [0u8; 4];
    let mut buffer_1 = [0u8; 4];
    let mut buffer_2 = [0u8; 4];

    file.read_exact(&mut buffer_0)?;
    file.read_exact(&mut buffer_1)?;
    file.read_exact(&mut buffer_2)?;

    let mut triangle = Triangle {
        v1: Vec3(0, 0, 0),
        v2: Vec3(0, 0, 0),
        v3: Vec3(0, 0, 0),
    };

    let x1 = i32::from_le_bytes(buffer_0);
    let y1 = i32::from_le_bytes(buffer_1);
    let z1 = u32::from_le_bytes(buffer_2);

    let xt = x1 & 0x3fff;
    triangle.v1.0 = xt << 4;
    triangle.v2.0 = (((x1 << 9) >> 0x17) + xt) * 0x10;
    triangle.v3.0 = ((x1 >> 0x17) + xt) * 0x10;

    let yt = y1 & 0x3fff;
    triangle.v1.1 = yt << 4;
    triangle.v2.1 = (((y1 << 9) >> 0x17) + yt) * 0x10;
    triangle.v3.1 = ((y1 >> 0x17) + yt) * 0x10;

    let zt = z1 & 0x3fff;
    triangle.v1.2 = (zt << 4) as i32;
    triangle.v2.2 = ((((z1 << 8) >> 0x18) + zt) * 0x10) as i32;
    triangle.v3.2 = (((z1 >> 0x18) + zt) * 0x10) as i32;

    Ok(Triangle {
        v1: triangle.v1,
        v2: triangle.v2,
        v3: triangle.v3,
    })
}

pub fn convert(file_pb: PathBuf) -> anyhow::Result<()> {
    let mut file = File::open(&file_pb)?;

    let mut dst = file_pb.clone();
    dst.set_extension("obj");

    let mut dst_file = File::create(dst)?;

    let mut face_counter = 0;
    loop {
        let triangle = match read_tri(&mut file) {
            Ok(tri) => tri,
            Err(_) => break,
        };

        dst_file.write(
            format!(
                "v {} {} {}\n",
                triangle.v1.0 as f32 / 4096.0,
                triangle.v1.2 as f32 / 4096.0,
                triangle.v1.1 as f32 / -4096.0
            )
            .as_bytes(),
        )?;
        dst_file.write(
            format!(
                "v {} {} {}\n",
                triangle.v2.0 as f32 / 4096.0,
                triangle.v2.2 as f32 / 4096.0,
                triangle.v2.1 as f32 / -4096.0
            )
            .as_bytes(),
        )?;
        dst_file.write(
            format!(
                "v {} {} {}\n",
                triangle.v3.0 as f32 / 4096.0,
                triangle.v3.2 as f32 / 4096.0,
                triangle.v3.1 as f32 / -4096.0
            )
            .as_bytes(),
        )?;

        face_counter += 1;
    }

    for i in 0..face_counter {
        dst_file.write(format!("f {} {} {}\n", i * 3 + 1, i * 3 + 2, i * 3 + 3).as_bytes())?;
    }

    Ok(())
}
