use image::{ImageBuffer, Rgba};

use crate::texture_processing::*;

pub fn processed_block_texture(name: &str) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let auto_block_texture = || block_texture(name);
    match name {
        "grass_block" => full_isometric_sides(
            &biome_tint(&block_texture("grass_block_top")),
            &block_texture("grass_block_side"),
        ),
        "oak_leaves" | "birch_leaves" | "acacia_leaves" | "jungle_leaves" | "dark_oak_leaves"
        | "spruce_leaves" => full_isometric(&biome_tint(&auto_block_texture())),
        "grass" => crossed_planes(&biome_tint(&auto_block_texture())),
        "dandelion" => crossed_planes(&auto_block_texture()),
        "water" => full_isometric_sides(
            &crop16(&tint(&block_texture("water_still"), (0, 0, 255))),
            &crop16(&tint(&block_texture("water_flowing"), (0, 0, 255))),
        ),
        _ => {
            println!("Found some '{}', that has no special case.", name);
            full_isometric(&auto_block_texture())
        }
    }
}
