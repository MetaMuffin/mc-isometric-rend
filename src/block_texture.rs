

use image::{ImageBuffer, Rgba};

use crate::texture_processing::*;


pub fn processed_block_texture(name: &str) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let auto_block_texture = || block_texture(name);
    match name {
        "grass_block" => biome_tint(auto_block_texture()),
        _ => block_texture("debug")
    }
}
