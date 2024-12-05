#![allow(clippy::too_many_arguments)]

use framebuffer::{Framebuffer, IndexMap};

#[derive(Debug, Clone)]
pub struct Sprite {
    width: u16,
    height: u16,
    pal_offset: u8,
    rle: bool,
    data: Vec<u8>,
}

impl Sprite {
    pub fn new_from_slice(data: &[u8]) -> Self {
        let w0 = u16::from_le_bytes(data[0..2].try_into().unwrap());
        let w1 = u16::from_le_bytes(data[2..4].try_into().unwrap());
        let data = Vec::from(&data[4..]);

        let flags = (w0 & 0xfe00) >> 8;
        let width = w0 & 0x01ff;
        let pal_offset = ((w1 & 0xff00) >> 8) as u8;
        let height = w1 & 0x00ff;

        let rle = (flags & 0x80) != 0;

        Sprite {
            width,
            height,
            pal_offset,
            rle,
            data,
        }
    }

    pub fn draw(
        &self,
        index: usize,
        x: usize,
        y: usize,
        flip_x: bool,
        flip_y: bool,
        scale: u8,
        pal_offset: u8,
        frame: &mut Framebuffer,
        index_map: &mut Option<IndexMap>,
    ) -> std::io::Result<()> {
        let mut pal_offset = pal_offset;
        if pal_offset == 0 {
            pal_offset = self.pal_offset();
        }

        blit::draw(
            index,
            x,
            y,
            self.width(),
            self.height(),
            self.rle(),
            flip_x,
            flip_y,
            scale,
            pal_offset,
            self.data(),
            frame,
            index_map,
        )
    }

    pub fn width(&self) -> usize {
        self.width as usize
    }

    pub fn height(&self) -> usize {
        self.height as usize
    }

    pub fn pal_offset(&self) -> u8 {
        self.pal_offset
    }

    pub fn set_pal_offset(&mut self, pal_offset: u8) {
        self.pal_offset = pal_offset;
    }

    pub fn rle(&self) -> bool {
        self.rle
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
