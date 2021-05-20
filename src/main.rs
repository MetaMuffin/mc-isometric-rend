#![feature(cstring_from_vec_with_nul)]
#![feature(str_split_once)]

use std::path::Path;

pub mod block_texture;
pub mod compositor;
pub mod texture_processing;
pub mod seg_parser;

fn main() {
    let files = std::fs::read_dir(&Path::new("./public/segments/")).unwrap();
    for s in files {
        let filename = String::from(s.unwrap().file_name().to_str().unwrap());
        println!("Rendering {}...", filename);
        compositor::render_segment(&filename);
    }
}
