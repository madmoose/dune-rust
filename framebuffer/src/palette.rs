use std::io::{Cursor, Seek};

use bytes_ext::ReadBytesExt;

#[derive(Debug, Clone)]
pub struct Palette([(u8, u8, u8); 256]);

fn scale_6bit_to_8bit(c: u8) -> u8 {
    (255 * (c as u16) / 63) as u8
}

impl Palette {
    pub fn new() -> Self {
        let mut pal = [(0u8, 0u8, 0u8); 256];

        for i in 0..256 {
            pal[i + 0] = (0, 0, 0);
        }

        Palette(pal)
    }

    pub fn clear(&mut self) {
        for i in 0..256 {
            self.set(i, (0, 0, 0));
        }
    }

    pub fn get(&self, i: usize) -> (u8, u8, u8) {
        self.0[i]
    }

    pub fn get_scaled(&self, i: usize) -> (u8, u8, u8) {
        let c = self.0[i];

        (
            scale_6bit_to_8bit(c.0),
            scale_6bit_to_8bit(c.1),
            scale_6bit_to_8bit(c.2),
        )
    }

    pub fn set(&mut self, i: usize, rgb: (u8, u8, u8)) {
        self.0[i] = rgb;
    }

    pub fn set_all(&mut self, pal: &[u8; 768]) {
        for i in 0..256 {
            self.set(i, (pal[3 * i + 0], pal[3 * i + 1], pal[3 * i + 2]))
        }
    }

    pub fn as_slice(&self) -> &[(u8, u8, u8); 256] {
        &self.0
    }

    pub fn as_mut_slice(&mut self) -> &mut [(u8, u8, u8); 256] {
        &mut self.0
    }

    pub fn apply_palette_update(&mut self, data: &[u8]) -> Result<u64, std::io::Error> {
        let mut r = Cursor::new(data);

        loop {
            let read_u8 = r.read_u8();
            let index = read_u8? as usize;
            let mut count = r.read_u8()? as usize;

            if index == 1 && count == 0 {
                r.seek_relative(3)?;
                continue;
            }
            if index == 0xff && count == 0xff {
                break;
            }
            if count == 0 {
                count = 256;
            }

            for i in 0..count {
                let cr = r.read_u8()?;
                let cg = r.read_u8()?;
                let cb = r.read_u8()?;

                if index + i <= 255 {
                    self.set(index + i, (cr, cg, cb));
                }
            }
        }

        loop {
            match r.read_u8() {
                Ok(0xff) => {}
                Ok(_) => {
                    r.seek_relative(-1)?;
                    break;
                }
                Err(_) => {
                    break;
                }
            }
        }

        Ok(r.position())
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}
