#![feature(cstring_from_vec_with_nul)]
use byteorder::{NativeEndian, ReadBytesExt};
use image::{DynamicImage, GenericImageView};
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
    let out_size = TEX_RESOLUTION * SEG_SIZE as u32;
 
    let zseg_file = File::open(&Path::new("./input/seg.1.2.zseg")).unwrap();
    let mut zseg = BufReader::new(zseg_file);

    let mut top = vec![[[0xFFFFu16; SEG_SIZE]; SEG_SIZE]; 255];

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
                top[(c / SEG_SIZE / SEG_SIZE)][c % SEG_SIZE][(c / SEG_SIZE) % SEG_SIZE] = block_id;
                c += 1
            }
            Err(_e) => break,
        }
    }

    for (i,layer) in top.iter().take(100).enumerate() {
        println!("layer {}", i);
        let mut imgbuf = image::ImageBuffer::new(out_size, out_size);
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let pos = (x / TEX_RESOLUTION, y / TEX_RESOLUTION);
            let block_id = layer[pos.0 as usize][pos.1 as usize];
            let tex = textures
                .get(&block_id)
                .expect("Found block that is not in the palette");
            *pixel = tex.get_pixel(x % TEX_RESOLUTION, y % TEX_RESOLUTION)
        }
        imgbuf.save(format!("public/generated/layer-{}.png", i).as_str()).unwrap();
    }

}
