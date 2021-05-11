#![feature(cstring_from_vec_with_nul)]
use byteorder::{NativeEndian, ReadBytesExt};
use image::Rgba;
use image::{GenericImageView, ImageBuffer};
use imageproc::geometric_transformations::Projection;
use integer_encoding::VarIntReader;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

const SEG_SIZE: usize = 16 * 8;

fn main() {
    render_segment("seg.1.2");
}

pub fn render_segment(name: &str) {
    let mut seg = vec![[[0xFFFFu16; SEG_SIZE]; SEG_SIZE]; 255];
    let mut textures: HashMap<u16, ImageBuffer<Rgba<u8>, Vec<u8>>> = HashMap::new();

    {
        let zseg_file = File::open(&Path::new(
            format!("./public/segments/{}.zseg", name).as_str(),
        ))
        .unwrap();
        let mut zseg = BufReader::new(zseg_file);
        loop {
            let mut block_name_raw = vec![];
            zseg.read_until(0x00, &mut block_name_raw).unwrap();
            block_name_raw.pop();
            let block_id = zseg.read_u16::<NativeEndian>().unwrap();
            if block_name_raw.len() == 0 {
                break;
            };
            let block_name = String::from_utf8(block_name_raw).unwrap();

            let texture = image::open(&Path::new(
                format!("./res/assets/minecraft/textures/block/{}.png", block_name).as_str(),
            ))
            .unwrap_or_else(|_err| {
                image::open(&Path::new(
                    format!("./res/assets/minecraft/textures/block/debug.png").as_str(),
                ))
                .expect("Could not even load the fallback texture")
            })
            .into_rgba8();
            let isometric = generate_isometric_block_texture(&texture);
            textures.insert(block_id, isometric);
        }
        let mut c = 0;
        loop {
            match zseg.read_varint::<usize>() {
                Ok(dist) => {
                    c += dist;
                    let block_id = zseg.read_u16::<NativeEndian>().unwrap() & 0x7fff;
                    seg[(c / SEG_SIZE / SEG_SIZE)][c % SEG_SIZE][(c / SEG_SIZE) % SEG_SIZE] =
                        block_id;
                    c += 1
                }
                Err(_e) => break,
            }
        }
    }

    let mut view: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::new(16 * SEG_SIZE as u32, 16 * SEG_SIZE as u32);

    for y in 0..255usize {
        for x in 0..SEG_SIZE {
            for z in 0..SEG_SIZE {
                match seg[y][x][z] {
                    0xFFFF => (),
                    103 => (),
                    block_id => {
                        println!("{}-{}-{}: {}\x1b[A\r", x, y, z, block_id);
                        let coords = isometric_coord_mapping(x, y, z);
                        let texture = textures.get(&block_id).expect("asdasd");
                        image_buffer_blit(&mut view, texture, coords);
                    }
                }
            }
        }
    }

    view.save(format!("public/prerendered/{}.png", name))
        .unwrap();
}

pub fn image_buffer_blit(
    target: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    offset: (u32, u32),
) {
    for (x, y, source_pixel) in source.enumerate_pixels() {
        if target.in_bounds(x + offset.0, y + offset.1) {
            let target_pixel = target.get_pixel_mut(x + offset.0, y + offset.1);
            let new_pixel = Rgba::from([
                (target_pixel.0[0] * (255 - source_pixel.0[3])) / 255
                    + (source_pixel.0[0] * source_pixel.0[3]) / 255,
                (target_pixel.0[1] * (255 - source_pixel.0[3])) / 255
                    + (source_pixel.0[1] * source_pixel.0[3]) / 255,
                (target_pixel.0[2] * (255 - source_pixel.0[3])) / 255
                    + (source_pixel.0[2] * source_pixel.0[3]) / 255,
                255,
            ]);
            *target_pixel = new_pixel;
        }
    }
}

pub fn isometric_coord_mapping(x: usize, y: usize, z: usize) -> (u32, u32) {
    let (x, y, z) = (x as i32, y as i32, z as i32);
    let (x, y, z) = (x * 16, y * 16, z * 16);
    let (sx, sy) = (y + (x + z) / 2, (SEG_SIZE * 16) as i32 - (x + z) / 2);
    (sx as u32, sy as u32)
}

pub fn generate_isometric_block_texture(
    texture: &ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let projection_y = Projection::from_control_points(
        [(0.0, 0.0), (16.0, 0.0), (16.0, 16.0), (0.0, 16.0)],
        [(0.0, 4.0), (8.0, 0.0), (16.0, 4.0), (8.0, 8.0)],
    )
    .unwrap();
    let projection_x = Projection::from_control_points(
        [(0.0, 0.0), (16.0, 0.0), (0.0, 16.0), (16.0, 16.0)],
        [(0.0, 4.0), (8.0, 8.0), (0.0, 12.0), (8.0, 16.0)],
    )
    .unwrap();
    let projection_z = Projection::from_control_points(
        [(0.0, 0.0), (16.0, 0.0), (0.0, 16.0), (16.0, 16.0)],
        [(8.0, 8.0), (16.0, 4.0), (8.0, 16.0), (16.0, 12.0)],
    )
    .unwrap();

    let face_x = imageproc::geometric_transformations::warp(
        &texture,
        &projection_x,
        imageproc::geometric_transformations::Interpolation::Nearest,
        Rgba::from([0, 0, 0, 0]),
    );
    let face_y = imageproc::geometric_transformations::warp(
        &texture,
        &projection_y,
        imageproc::geometric_transformations::Interpolation::Nearest,
        Rgba::from([0, 0, 0, 0]),
    );
    let face_z = imageproc::geometric_transformations::warp(
        &texture,
        &projection_z,
        imageproc::geometric_transformations::Interpolation::Nearest,
        Rgba::from([0, 0, 0, 0]),
    );

    composite_block_faces(&vec![face_x, face_y, face_z])
}

pub fn composite_block_faces(
    faces: &Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut buf = image::ImageBuffer::new(16, 16);
    for (x, y, pixel) in buf.enumerate_pixels_mut() {
        let composite: (u16, u16, u16, u16) = faces
            .iter()
            .map(|f| {
                let p = f.get_pixel(x, y).0;
                (p[0] as u16, p[1] as u16, p[2] as u16, p[3] as u16)
            })
            .fold((0, 0, 0, 0), |a, v| {
                (
                    (a.0 * (255 - v.3)) / 255 + (v.0 * v.3) / 255,
                    (a.1 * (255 - v.3)) / 255 + (v.1 * v.3) / 255,
                    (a.2 * (255 - v.3)) / 255 + (v.2 * v.3) / 255,
                    a.3 + v.3,
                )
            });
        *pixel = Rgba::from([
            composite.0.min(255) as u8,
            composite.1.min(255) as u8,
            composite.2.min(255) as u8,
            composite.3.min(255) as u8,
        ])
    }
    return buf;
}
