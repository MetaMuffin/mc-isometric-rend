#![feature(cstring_from_vec_with_nul)]
use byteorder::{NativeEndian, ReadBytesExt};
use image::{DynamicImage, GenericImageView, ImageBuffer};
use image::{Rgb, Rgba};
use integer_encoding::VarIntReader;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

const SEG_SIZE: usize = 16 * 8;
const TEX_RESOLUTION: u32 = 16;

fn main() {
    // let out_size = TEX_RESOLUTION * SEG_SIZE as u32;
    let out_size = 8196;

    let zseg_file = File::open(&Path::new("./input/seg.1.2.zseg")).unwrap();
    let mut zseg = BufReader::new(zseg_file);

    let mut seg = vec![[[0xFFFFu16; SEG_SIZE]; SEG_SIZE]; 255];

    let mut textures: HashMap<u16, DynamicImage> = HashMap::new();

    loop {
        let mut block_name_raw = vec![];
        zseg.read_until(0x00, &mut block_name_raw).unwrap();
        block_name_raw.pop();
        let block_id = zseg.read_u16::<NativeEndian>().unwrap();
        if block_name_raw.len() == 0 {
            break;
        };
        let block_name = String::from_utf8(block_name_raw).unwrap();
        println!("{}: {}", block_id, block_name);

        let texture = image::open(&Path::new(
            format!("./res/assets/minecraft/textures/block/{}.png", block_name).as_str(),
        ))
        .unwrap_or_else(|_err| {
            image::open(&Path::new(
                format!("./res/assets/minecraft/textures/block/debug.png").as_str(),
            ))
            .expect("Could not even load the fallback texture")
        });
        textures.insert(block_id, texture);
    }

    let mut c = 0;
    loop {
        match zseg.read_varint::<usize>() {
            Ok(dist) => {
                c += dist;
                let block_id = zseg.read_u16::<NativeEndian>().unwrap() & 0x7fff;
                seg[(c / SEG_SIZE / SEG_SIZE)][c % SEG_SIZE][(c / SEG_SIZE) % SEG_SIZE] = block_id;
                c += 1
            }
            Err(_e) => break,
        }
    }

    let mut imgbuf: ImageBuffer<Rgba<u8>, _> = image::ImageBuffer::new(out_size, out_size);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        // do some raytracing because muffins dont know how to do projection stuff
        if x == 0 {
            println!("{}\x1b[A\r", y);
        }
        let (mut x, mut y, out_size, seg_size) =
            (x as f64, y as f64, out_size as f64, SEG_SIZE as f64);
        y /= out_size;
        x /= out_size;

        y = 1.0 - y;
        x *= 2.0;
        x -= 1.0;

        y *= 255.0;
        x *= seg_size;

        let mut ray = (
            seg_size + seg_size - x - y,
            seg_size + y + x,
            seg_size + seg_size + x - y,
        );
        let ray_dir = (-0.5, -0.5, -0.5);
        for _ in 0..(1 << 12) {
            let block = if ray.0 >= 0.0
                && ray.0 < seg_size
                && ray.2 >= 0.0
                && ray.2 < seg_size
                && ray.1 >= 0.0
                && ray.1 < 255.0
            {
                let map_coord = (ray.0 as usize, ray.1 as usize, ray.2 as usize);
                seg[map_coord.1][map_coord.0][map_coord.2]
            } else {
                0xFFFF
            };
            match block {
                0xFFFF => {
                    ray.0 += ray_dir.0;
                    ray.1 += ray_dir.1;
                    ray.2 += ray_dir.2;
                }
                103 => {
                    println!("This should not happen");
                    break;
                }
                block_id => {
                    let map_coord_off = ((ray.0 + ray.1) % 1.0, (ray.0 + ray.2) % 1.0);
                    let tex = textures
                        .get(&block_id)
                        .expect("Found block that is not in the palette");
                    *pixel = tex.get_pixel(
                        (map_coord_off.0 * TEX_RESOLUTION as f64) as u32,
                        (map_coord_off.1 * TEX_RESOLUTION as f64) as u32,
                    );
                    break;
                }
            }
        }
    }

    imgbuf.save(format!("public/generated/view.png")).unwrap();
}
