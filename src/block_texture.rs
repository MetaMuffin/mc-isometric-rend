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
        "dandelion" | "orange_tulip" | "azure_bluet" | "allium" | "poppy" | "cornflower" => {
            crossed_planes(&auto_block_texture())
        }
        "lilac" | "peony" | "rose_bush" | "tall_grass" => crossed_planes(&auto_block_texture()),
        "water" => full_isometric_sides(
            &crop16(&tint(&block_texture("water_still"), (0, 0, 255))),
            &crop16(&tint(&block_texture("water_flow"), (0, 0, 255))),
        ),
        "lava" => full_isometric_sides(
            &crop16(&block_texture("lava_still")),
            &crop16(&block_texture("lava_flow")),
        ),
        "vine" => full_isometric(&biome_tint(&auto_block_texture())),
        "lily_pad" => crossed_planes(&biome_tint(&auto_block_texture())),
        "sugar_cane" => crossed_planes(&auto_block_texture()),

        "removed" => full_isometric(&transparent()),
        _ => {
            // println!("{}", name);
            full_isometric(&auto_block_texture())
        }
    }
}
