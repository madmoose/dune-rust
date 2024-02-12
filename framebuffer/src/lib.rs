#![allow(clippy::identity_op)]

use std::{fs::File, io::Write};

pub struct Framebuffer<'a> {
    w: usize,
    h: usize,
    pal: [u8; 768],
    pixels: &'a mut [u8],
}

impl Framebuffer<'_> {
    pub fn new_with_pixel_data<'a>(
        w: usize,
        h: usize,
        pixels: &'a mut [u8],
        pal: &'a [u8; 768],
    ) -> Framebuffer<'a> {
        Framebuffer {
            w,
            h,
            pal: *pal,
            pixels,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.pixels
    }

    pub fn clear(&mut self) {
        for y in 0..self.h {
            for x in 0..self.w {
                self.put_pixel(x, y, 240);
            }
        }
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, c: u8) {
        let c = c as usize;
        self.pixels[4 * (self.w * y + x) + 0] = self.pal[3 * c + 0];
        self.pixels[4 * (self.w * y + x) + 1] = self.pal[3 * c + 1];
        self.pixels[4 * (self.w * y + x) + 2] = self.pal[3 * c + 2];
        self.pixels[4 * (self.w * y + x) + 3] = 255;
    }

    pub fn write_ppm(&self, filename: &str) -> std::io::Result<()> {
        let mut data = vec![0; 3 * self.w * self.h];

        for y in 0..self.h {
            for x in 0..self.w {
                data[3 * (y * self.w + x) + 0] = self.pixels[4 * (y * self.w + x) + 0];
                data[3 * (y * self.w + x) + 1] = self.pixels[4 * (y * self.w + x) + 1];
                data[3 * (y * self.w + x) + 2] = self.pixels[4 * (y * self.w + x) + 2];
            }
        }

        let mut f = File::create(filename)?;
        writeln!(f, "P6 {} {} 255", self.w, self.h)?;
        f.write_all(&data)?;
        Ok(())
    }
}
