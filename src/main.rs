#![feature(cstring_from_vec_with_nul)]
#![feature(str_split_once)]

pub mod texture_processing;
pub mod block_texture;
pub mod compositor;

fn main() {
    compositor::render_segment("seg.1.2");
}
