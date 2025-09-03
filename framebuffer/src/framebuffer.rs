use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::Palette;

#[derive(Clone)]
pub struct Framebuffer {
    w: usize,
    h: usize,
    is_set: Vec<bool>,
    pixels: Vec<u8>,
}

impl Framebuffer {
    pub fn new(w: usize, h: usize) -> Framebuffer {
        let pixels = vec![0u8; w * h];
        let is_set = vec![false; w * h];
        Framebuffer {
            w,
            h,
            is_set,
            pixels,
        }
    }

    pub fn clear(&mut self) {
        for y in 0..self.h {
            for x in 0..self.w {
                self.put_pixel(x, y, 0);
                self.is_set[y * self.w + x] = false;
            }
        }
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, c: u8) {
        if x >= self.w || y >= self.h {
            return;
        }
        self.pixels[y * self.w + x] = c;
        self.is_set[y * self.w + x] = true;
    }

    pub fn get_pixel(&mut self, x: usize, y: usize) -> u8 {
        self.pixels[y * self.w + x]
    }

    pub fn get_is_set(&mut self, x: usize, y: usize) -> bool {
        self.is_set[y * self.w + x]
    }

    pub fn write_ppm(&self, pal: &Palette, filename: &str) -> std::io::Result<()> {
        let mut data = vec![0; 3 * self.w * self.h];

        for y in 0..self.h {
            for x in 0..self.w {
                let i = self.pixels[y * self.w + x];
                let p = pal.get_scaled(i as usize);
                data[3 * (y * self.w + x) + 0] = p.0;
                data[3 * (y * self.w + x) + 1] = p.1;
                data[3 * (y * self.w + x) + 2] = p.2;
            }
        }

        let mut f = File::create(filename)?;
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
        let dst_width = self.w * scale_x;
        let dst_height = self.h * scale_y;

        let mut data = vec![0; 3 * dst_width * dst_height];

        for y in 0..dst_height {
            let y0 = y / scale_y;
            for x in 0..dst_width {
                let x0 = x / scale_x;

                let i = self.pixels[y0 * self.w + x0];
                let p = pal.get_scaled(i as usize);

                data[3 * (y * dst_width + x) + 0] = p.0;
                data[3 * (y * dst_width + x) + 1] = p.1;
                data[3 * (y * dst_width + x) + 2] = p.2;
            }
        }

        let mut f = File::create(filename)?;
        writeln!(f, "P6 {} {} 255", self.w * scale_x, self.h * scale_y)?;
        f.write_all(&data)?;
        Ok(())
    }

    pub fn write_png_scaled<P: AsRef<Path>>(&self, pal: &Palette, path: P) -> std::io::Result<()> {
        self.write_png_scaled_(pal, path.as_ref())
    }

    fn write_png_scaled_(&self, pal: &Palette, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let w = &mut BufWriter::new(file);

        let expanded_width = 5 * self.w;
        let expanded_height = 6 * self.h;
        let mut rgba_data = vec![0u8; expanded_width * expanded_height * 4];

        for y in 0..expanded_height {
            for x in 0..expanded_width {
                let src_idx = (y / 6) * self.w + (x / 5);
                let c = self.pixels[src_idx] as usize;
                let rgb = pal.get_scaled(c);
                rgba_data[4 * (y * expanded_width + x) + 0] = rgb.0;
                rgba_data[4 * (y * expanded_width + x) + 1] = rgb.1;
                rgba_data[4 * (y * expanded_width + x) + 2] = rgb.2;
                rgba_data[4 * (y * expanded_width + x) + 3] = self.is_set[src_idx] as u8 * 255;
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
