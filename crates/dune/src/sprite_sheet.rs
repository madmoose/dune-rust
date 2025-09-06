use std::io::Cursor;

use bytes_ext::ReadBytesExt;

use crate::{Palette, hsq, sprite::Sprite};

enum SpriteOrData {
    Sprite(Sprite),
    Data(Vec<u8>),
}

pub struct SpriteSheet {
    pal_update: Option<Vec<u8>>,
    resource_count: u16,
    sprites: Vec<SpriteOrData>,
}

impl SpriteSheet {
    pub fn from_possibly_compressed_slice(data: &[u8]) -> Result<Self, std::io::Error> {
        let mut reader = Cursor::new(data);
        let header = hsq::Header::from_reader(&mut reader)?;

        if !header.is_compressed() {
            return SpriteSheet::from_slice(data);
        }

        if header.compressed_size() as usize != data.len() {
            println!("Packed length does not match resource size");
            return SpriteSheet::from_slice(data);
        }

        let mut unpacked_data = vec![0; header.uncompressed_size() as usize];
        let mut writer = Cursor::new(&mut unpacked_data);

        hsq::unhsq(reader, &mut writer)?;

        SpriteSheet::from_slice(&unpacked_data)
    }

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

        let resource_count = offsets.len() as u16;
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

    pub fn resource_count(&self) -> u16 {
        self.resource_count
    }

    pub fn get_sprite(&self, id: u16) -> Option<&Sprite> {
        match self.sprites.get(id as usize) {
            Some(SpriteOrData::Sprite(sprite)) => Some(sprite),
            Some(_) => None,
            None => None,
        }
    }

    pub fn get_resource(&self, id: u16) -> Option<&[u8]> {
        match self.sprites.get(id as usize) {
            Some(SpriteOrData::Data(data)) => Some(data),
            Some(_) => None,
            None => None,
        }
    }
}
