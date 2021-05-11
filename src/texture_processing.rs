use image::{ImageBuffer, Rgba};
use imageproc::geometric_transformations::Projection;
use std::path::Path;

pub type Texture = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub fn biome_tint(source: Texture) -> Texture {
    let biome_tint_color = (0, 255, 10);
    let mut t = ImageBuffer::new(16, 16);
    for (x, y, p) in t.enumerate_pixels_mut() {
        let sp = source.get_pixel(x, y);
        *p = Rgba::from([
            sp.0[0] * biome_tint_color.0,
            sp.0[1] * biome_tint_color.1,
            sp.0[2] * biome_tint_color.2,
            sp.0[3],
        ])
    }
    return t;
}

pub fn generate_isometric_block_texture(block_name: &str) -> Texture {
    let texture = block_texture(block_name);
    make_full_block_texture_isometric(&texture, &texture)
}

pub fn block_texture(block_name: &str) -> Texture {
    image::open(&Path::new(
        format!("./res/assets/minecraft/textures/block/{}.png", block_name).as_str(),
    ))
    .unwrap_or_else(|_err| {
        image::open(&Path::new(
            format!("./res/assets/minecraft/textures/block/debug.png").as_str(),
        ))
        .expect("Could not even load the fallback texture")
    })
    .into_rgba8()
}

pub fn make_full_block_texture_isometric(top: &Texture, side: &Texture) -> Texture {
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
        &side,
        &projection_x,
        imageproc::geometric_transformations::Interpolation::Nearest,
        Rgba::from([0, 0, 0, 0]),
    );
    let face_y = imageproc::geometric_transformations::warp(
        &top,
        &projection_y,
        imageproc::geometric_transformations::Interpolation::Nearest,
        Rgba::from([0, 0, 0, 0]),
    );
    let face_z = imageproc::geometric_transformations::warp(
        &side,
        &projection_z,
        imageproc::geometric_transformations::Interpolation::Nearest,
        Rgba::from([0, 0, 0, 0]),
    );

    composite_block_faces(&vec![face_x, face_y, face_z])
}

pub fn composite_block_faces(faces: &Vec<Texture>) -> Texture {
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
