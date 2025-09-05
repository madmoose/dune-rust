use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::{Palette, image::Image};

pub type Framebuffer = Image<u8>;

impl Framebuffer {
    pub fn write_ppm(&self, pal: &Palette, filename: &str) -> std::io::Result<()> {
        let width = self.w as usize;
        let height = self.h as usize;
        let mut data = Vec::with_capacity(3 * width * height);

        for &pixel in &self.pixels {
            let rgb = pal.get_rgb888(pixel as usize);
            data.push(rgb.0);
            data.push(rgb.1);
            data.push(rgb.2);
        }

        let mut f = BufWriter::new(File::create(filename)?);
        writeln!(f, "P6 {} {} 255", self.w, self.h)?;
        f.write_all(&data)?;
        Ok(())
    }

    pub fn write_ppm_scaled(
        &self,
        pal: &Palette,
        filename: &str,
        scale_x: usize,
        scale_y: usize,
    ) -> std::io::Result<()> {
        let src_width = self.w as usize;
        let dst_width = src_width * scale_x;
        let dst_height = self.h as usize * scale_y;

        let mut data = Vec::with_capacity(3 * dst_width * dst_height);

        for src_y in 0..self.h as usize {
            let src_row = &self.pixels[src_y * src_width..(src_y + 1) * src_width];

            for _ in 0..scale_y {
                for &pixel in src_row {
                    let rgb = pal.get_rgb888(pixel as usize);
                    for _ in 0..scale_x {
                        data.push(rgb.0);
                        data.push(rgb.1);
                        data.push(rgb.2);
                    }
                }
            }
        }

        let mut f = BufWriter::new(File::create(filename)?);
        writeln!(f, "P6 {} {} 255", dst_width, dst_height)?;
        f.write_all(&data)?;
        Ok(())
    }

    pub fn write_png_scaled<P: AsRef<Path>>(&self, pal: &Palette, path: P) -> std::io::Result<()> {
        self.write_png_scaled_(pal, path.as_ref())
    }

    fn write_png_scaled_(&self, pal: &Palette, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let w = &mut BufWriter::new(file);

        let expanded_width = 5 * self.w as usize;
        let expanded_height = 6 * self.h as usize;
        let mut rgba_data = vec![0u8; expanded_width * expanded_height * 4];

        for y in 0..expanded_height {
            for x in 0..expanded_width {
                let src_idx = (y / 6) * self.w as usize + (x / 5);
                let c = self.pixels[src_idx] as usize;
                let rgb = pal.get_rgb888(c);
                rgba_data[4 * (y * expanded_width + x) + 0] = rgb.0;
                rgba_data[4 * (y * expanded_width + x) + 1] = rgb.1;
                rgba_data[4 * (y * expanded_width + x) + 2] = rgb.2;
                // rgba_data[4 * (y * expanded_width + x) + 3] = self.is_set[src_idx] as u8 * 255;
            }
        }

        let mut encoder = png::Encoder::new(w, expanded_width as u32, expanded_height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&rgba_data)?;

        Ok(())
    }
}
