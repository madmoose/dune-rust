#![allow(clippy::identity_op)]
#![feature(file_buffered)]

mod pal;

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

pub use pal::Pal;

pub struct Framebuffer<'a> {
    w: usize,
    h: usize,
    pal: Pal,
    pixels: &'a mut [u8],
}

#[derive(Debug)]
pub struct IndexMap(pub [u8; 320 * 200]);

impl IndexMap {
    pub fn new() -> IndexMap {
        IndexMap([255; 320 * 200])
    }

    pub fn clear(&mut self) {
        self.0.fill(0xff);
    }

    pub fn set_index(&mut self, x: usize, y: usize, index: usize) {
        if x < 320 && y < 200 {
            self.0[y * 320 + x] = index as u8;
        }
    }

    pub fn get_index(&self, x: usize, y: usize) -> Option<usize> {
        if x < 320 && y < 200 {
            let v = self.0[y * 320 + x];
            if v < 255 {
                return Some(v as usize);
            }
        }
        None
    }
}

impl Default for IndexMap {
    fn default() -> Self {
        Self::new()
    }
}

fn scale_6bit_to_8bit(c: u8) -> u8 {
    (255 * (c as u16) / 63) as u8
}

impl Framebuffer<'_> {
    pub fn new_with_pixel_data(w: usize, h: usize, pixels: &mut [u8]) -> Framebuffer<'_> {
        Framebuffer {
            w,
            h,
            pal: Pal::new(),
            pixels,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.pixels
    }

    pub fn pal(&self) -> &Pal {
        &self.pal
    }

    pub fn mut_pal(&mut self) -> &mut Pal {
        &mut self.pal
    }

    pub fn clear(&mut self) {
        for y in 0..self.h {
            for x in 0..self.w {
                self.put_pixel(x, y, 0);
            }
        }
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, c: u8) {
        if x >= self.w || y >= self.h {
            return;
        }
        self.pixels[y * self.w + x] = c;
    }

    pub fn get_rgb(&mut self, x: usize, y: usize) -> (u8, u8, u8) {
        let i = self.pixels[y * self.w + x];
        let mut p = self.pal.get(i as usize);

        p.0 = scale_6bit_to_8bit(p.0);
        p.1 = scale_6bit_to_8bit(p.1);
        p.2 = scale_6bit_to_8bit(p.2);
        p
    }

    pub fn write_ppm(&self, filename: &str) -> std::io::Result<()> {
        let mut data = vec![0; 3 * self.w * self.h];

        for y in 0..self.h {
            for x in 0..self.w {
                let i = self.pixels[y * self.w + x];
                let p = self.pal.get(i as usize);
                data[3 * (y * self.w + x) + 0] = scale_6bit_to_8bit(p.0);
                data[3 * (y * self.w + x) + 1] = scale_6bit_to_8bit(p.1);
                data[3 * (y * self.w + x) + 2] = scale_6bit_to_8bit(p.2);
            }
        }

        let mut f = File::create(filename)?;
        writeln!(f, "P6 {} {} 255", self.w, self.h)?;
        f.write_all(&data)?;
        Ok(())
    }

    pub fn write_pal_as_header(&self, filename: &str) -> std::io::Result<()> {
        let mut f = File::create_buffered(filename)?;

        writeln!(f, "uint8_t pal[768] = {{").unwrap();
        for i in 0..256 {
            let c = self.pal().get(i);
            write!(f, "\t{:3}, {:3}, {:3}", c.0, c.1, c.2)?;
            if i < 255 {
                write!(f, ",")?;
            }
            writeln!(f)?;
        }
        writeln!(f, "}};")?;

        Ok(())
    }

    pub fn write_ppm_scaled(
        &self,
        filename: &str,
        scale_x: usize,
        scale_y: usize,
    ) -> std::io::Result<()> {
        let dst_width = self.w * scale_x;
        let dst_height = self.h * scale_y;

        let mut data = vec![0; 3 * dst_width * dst_height];

        fn scale_6bit_to_8bit(c: u8) -> u8 {
            (255 * (c as u16) / 63) as u8
        }

        for y in 0..dst_height {
            let y0 = y / scale_y;
            for x in 0..dst_width {
                let x0 = x / scale_x;

                let i = self.pixels[y0 * self.w + x0];
                let p = self.pal.get(i as usize);

                data[3 * (y * dst_width + x) + 0] = scale_6bit_to_8bit(p.0);
                data[3 * (y * dst_width + x) + 1] = scale_6bit_to_8bit(p.1);
                data[3 * (y * dst_width + x) + 2] = scale_6bit_to_8bit(p.2);
            }
        }

        let mut f = File::create(filename)?;
        writeln!(f, "P6 {} {} 255", self.w * scale_x, self.h * scale_y)?;
        f.write_all(&data)?;
        Ok(())
    }

    pub fn write_png_scaled(&self, filename: &str) -> std::io::Result<()> {
        let path = Path::new(&filename);
        let file = File::create(path)?;
        let w = &mut BufWriter::new(file);

        fn scale_6bit_to_8bit(c: u8) -> u8 {
            (255 * (c as u16) / 63) as u8
        }

        let expanded_width = 5 * self.w;
        let expanded_height = 6 * self.h;
        let mut rgba_data = vec![0u8; expanded_width * expanded_height * 4];

        for y in 0..expanded_height {
            for x in 0..expanded_width {
                let c = self.pixels[(y / 6) * self.w + (x / 5)] as usize;
                let rgb = self.pal.get(c);
                rgba_data[4 * (y * expanded_width + x) + 0] = scale_6bit_to_8bit(rgb.0);
                rgba_data[4 * (y * expanded_width + x) + 1] = scale_6bit_to_8bit(rgb.1);
                rgba_data[4 * (y * expanded_width + x) + 2] = scale_6bit_to_8bit(rgb.2);
                rgba_data[4 * (y * expanded_width + x) + 3] = 255;
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
