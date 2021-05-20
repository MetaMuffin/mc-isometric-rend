use crate::block_texture::processed_block_texture;
use byteorder::{NativeEndian, ReadBytesExt};
use image::Rgba;
use image::{GenericImageView, ImageBuffer};
use integer_encoding::VarIntReader;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

pub const SEG_SIZE: usize = 16 * 8;
pub const CHUNK_HEIGHT: usize = 255;

pub fn render_segment(name: &str) {
    // has to be a vector because otherwise it is bigger than the stack. could maybe
    // be a box of an array instead, if that is faster
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

            let isometric = processed_block_texture(&block_name.as_str());
            textures.insert(block_id, isometric);
        }
        let mut c = 0;
        loop {
            match zseg.read_varint::<usize>() {
                Ok(dist) => {
                    c += dist;
                    let block_id = zseg.read_u16::<NativeEndian>().unwrap() & 0x7fff;
                    let (x, y, z) = zseg_index_to_xyz(c);
                    seg[y as usize][x as usize][z as usize] = block_id;
                    c += 1;
                }
                Err(_e) => break,
            }
        }
    }

    let mut view: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(
        16 * SEG_SIZE as u32,
        16 * (SEG_SIZE as u32 / 2 + CHUNK_HEIGHT as u32),
    );

    for y in 0..CHUNK_HEIGHT {
        for x in 0..SEG_SIZE {
            for z in 0..SEG_SIZE {
                match seg[y][x][z] {
                    0xFFFF => (),
                    103 => (),
                    block_id => {
                        // println!("{}-{}-{}: {}\x1b[A\r", x, y, z, block_id);
                        let coords = isometric_coord_mapping(
                            x as i32, y as i32, z as i32
                        );
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
            // TODO linear alpha stuff
            // let new_pixel = Rgba::from([
            //     (target_pixel.0[0] * (255 - source_pixel.0[3])) / 255
            //         + (source_pixel.0[0] * source_pixel.0[3]) / 255,
            //     (target_pixel.0[1] * (255 - source_pixel.0[3])) / 255
            //         + (source_pixel.0[1] * source_pixel.0[3]) / 255,
            //     (target_pixel.0[2] * (255 - source_pixel.0[3])) / 255
            //         + (source_pixel.0[2] * source_pixel.0[3]) / 255,
            //     target_pixel.0[3].max(source_pixel.0[3]),
            // ]);
            let new_pixel = match source_pixel.0[3] > 128 {
                true => source_pixel.clone(),
                false => target_pixel.clone(),
            };
            *target_pixel = new_pixel;
        }
    }
}

const fn isometric_coord_mapping(x: i32, y: i32, z: i32) -> (u32, u32) {
    const BASE_X: i32 = 1020;
    const BASE_Y: i32 = 4072;

    const XDIFF: (i32, i32) = (-8,  4);
    const ZDIFF: (i32, i32) = ( 8,  4);
    const YDIFF: (i32, i32) = ( 0, -8);

    let diff = (XDIFF.0 * x + YDIFF.0 * y + ZDIFF.0 * z,
                XDIFF.1 * x + YDIFF.1 * y + ZDIFF.1 * z);

    let coords = (BASE_X + diff.0, BASE_Y + diff.1);

    (coords.0 as u32, coords.1 as u32)
}

const fn zseg_index_to_xyz(n: usize) -> (i32, i32, i32) {
    let (x, z, y) = (
        n % SEG_SIZE,
       (n / SEG_SIZE) % SEG_SIZE,
        n / SEG_SIZE / SEG_SIZE
    );

    (x as i32, y as i32, z as i32)
}
