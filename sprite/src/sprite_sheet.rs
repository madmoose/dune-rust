use std::io::Cursor;

use bytes_ext::ReadBytesExt;
use dune::Rect;
use framebuffer::{Framebuffer, Palette};

use crate::sprite::Sprite;

enum SpriteOrData {
    Sprite(Sprite),
    Data(Vec<u8>),
}

pub struct SpriteSheet {
    pal_update: Option<Vec<u8>>,
    resource_count: usize,
    sprites: Vec<SpriteOrData>,
}

impl SpriteSheet {
    pub fn from_slice(data: &[u8]) -> Result<Self, std::io::Error> {
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

        let resource_count = offsets.len();
        for offset in offsets {
            let (ofs, size) = offset;
            let slice = &data[ofs..ofs + size];
            if let Some(sprite) = Sprite::from_slice(slice) {
                sprites.push(SpriteOrData::Sprite(sprite));
            } else {
                sprites.push(SpriteOrData::Data(slice.to_vec()));
            }
        }

        Ok(SpriteSheet {
            pal_update,
            resource_count,
            sprites,
        })
    }

    pub fn apply_palette_update(&self, pal: &mut Palette) -> Result<(), std::io::Error> {
        let Some(data) = &self.pal_update else {
            return Ok(());
        };

        pal.apply_palette_update(data)?;

        Ok(())
    }

    pub fn resource_count(&self) -> usize {
        self.resource_count
    }

    pub fn get_sprite(&self, id: usize) -> Option<&Sprite> {
        match self.sprites.get(id) {
            Some(SpriteOrData::Sprite(sprite)) => Some(sprite),
            Some(_) => None,
            None => None,
        }
    }

    pub fn get_resource(&self, id: usize) -> Option<&[u8]> {
        match self.sprites.get(id) {
            Some(SpriteOrData::Data(data)) => Some(data),
            Some(_) => None,
            None => None,
        }
    }

    pub fn draw_sprite(&self, id: usize, x: i16, y: i16, framebuffer: &mut Framebuffer) {
        self.get_sprite(id)
            .unwrap()
            .draw(x, y, framebuffer)
            .unwrap();
    }

    pub fn draw_sprite_clipped(
        &self,
        id: usize,
        x: i16,
        y: i16,
        clip_rect: Rect,
        framebuffer: &mut Framebuffer,
    ) {
        let sprite = self.get_sprite(id).unwrap();
        sprite
            .draw_with_options(
                id,
                x,
                y,
                clip_rect,
                false,
                false,
                0,
                0,
                framebuffer,
                &mut None,
            )
            .unwrap();
    }
}
