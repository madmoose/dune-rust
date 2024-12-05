use std::io::{Cursor, Seek};

use bytes_ext::ReadBytesExt;
use framebuffer::Pal;

use crate::sprite::Sprite;

pub struct SpriteSheet {
    pal_update: Option<Vec<u8>>,
    sprites: Vec<Sprite>,
}

impl SpriteSheet {
    pub fn new(data: &[u8]) -> Result<Self, std::io::Error> {
        let size = data.len();

        let toc_pos = u16::from_le_bytes(data[0..2].try_into().unwrap()) as usize;
        let mut toc = Cursor::new(&data[toc_pos..]);

        let pal_update = if toc_pos <= 2 {
            None
        } else {
            Some(Vec::from(&data[2..toc_pos]))
        };

        let sprite_0_pos = toc.read_le_u16()? as usize;
        let sprite_count = sprite_0_pos / 2;

        let mut offsets = Vec::with_capacity(sprite_count);
        let mut prev_pos = sprite_0_pos;

        for _ in 1..sprite_count {
            let pos = toc.read_le_u16()? as usize;
            offsets.push((toc_pos + prev_pos, pos - prev_pos));
            prev_pos = pos;
        }
        offsets.push((toc_pos + prev_pos, size - toc_pos - prev_pos));

        let mut sprites = Vec::new();

        for offset in offsets {
            let (ofs, size) = offset;
            sprites.push(Sprite::new_from_slice(&data[ofs..ofs + size]));
        }

        Ok(SpriteSheet {
            pal_update,
            sprites,
        })
    }

    pub fn apply_palette_update(&self, pal: &mut Pal) -> Result<(), std::io::Error> {
        let Some(pal_update) = &self.pal_update else {
            return Ok(());
        };

        let mut r = Cursor::new(pal_update);

        loop {
            let index = r.read_u8()? as usize;
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

                pal.set(index + i, (cr, cg, cb));
            }
        }

        Ok(())
    }

    pub fn get_sprite(&self, id: usize) -> Option<&Sprite> {
        self.sprites.get(id)
    }
}
