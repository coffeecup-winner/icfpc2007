use std::io::{BufWriter, Write};

use crate::build::{Bitmap, Pixel, Position, RGB};

use png;

pub fn write_bitmap_as_png<W: Write>(bitmap: &Bitmap, out: W) -> std::io::Result<()> {
    let writer = BufWriter::new(out);
    
    let mut encoder = png::Encoder::new(writer, 600, 600);
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    let mut data = vec![0u8; 3 * 600 * 600];
    let mut i = 0;
    for y in 0..600 {
        for x in 0..600 {
            let Pixel { rgb: RGB(r, g, b), a: _ } = bitmap.get(Position(x, y));
            data[i] = r;
            data[i + 1] = g;
            data[i + 2] = b;
            i += 3;
        }
    }
    writer.write_image_data(&data)?;

    Ok(())
}
