use std::io::{Cursor, Seek};

use bytes_ext::ReadBytesExt;

use crate::{Framebuffer, Palette, blit, hnm::frame_header::FrameHeader, hsq};

pub struct HnmDecoder<'a> {
    data: &'a [u8],
    header_size: u16,
    frame_offsets: Vec<u32>,
    buffer: Vec<u8>,
}

impl<'a> HnmDecoder<'a> {
    pub fn new(data: &'a [u8], pal: &mut Palette) -> std::io::Result<Self> {
        let mut r = Cursor::new(data);
        let header_size = r.read_le_u16()?;

        let pal_size = pal.apply_palette_update(&data[2..])?;
        r.seek_relative(pal_size as i64)?;
        let toc_pos = r.position();

        let frame_count = (header_size as u64 - toc_pos) / 4;

        let mut frame_offsets = Vec::with_capacity(frame_count as usize);

        for _ in 0..frame_count {
            frame_offsets.push(r.read_le_u32()?);
        }

        let buffer = Vec::<u8>::new();

        Ok(HnmDecoder {
            data,
            header_size,
            frame_offsets,
            buffer,
        })
    }

    pub fn frame_count(&self) -> usize {
        self.frame_offsets.len() - 1
    }

    pub fn decode_frame(
        &mut self,
        frame: usize,
        framebuffer: &mut Framebuffer,
        pal: &mut Palette,
    ) -> std::io::Result<()> {
        let frame_pos = self.header_size as usize + self.frame_offsets[frame] as usize;

        println!("decode_frame({frame}): frame_pos = {frame_pos:x}");

        let mut r = Cursor::new(&self.data[frame_pos..]);
        let frame_size = r.read_le_u16()?;

        let frame_end = frame_pos + frame_size as usize;
        let mut r = Cursor::new(&self.data[frame_pos + 2..frame_end]);

        const BLOCK_TYPE_SD: u16 = 0x7364;
        const BLOCK_TYPE_PL: u16 = 0x706C;

        loop {
            let block_type = r.read_be_u16()?;

            match block_type {
                BLOCK_TYPE_SD => {
                    let block_size = r.read_le_u16()?;
                    r.seek_relative(block_size as i64 - 4)?;
                }
                BLOCK_TYPE_PL => {
                    let block_size = r.read_le_u16()?;
                    let pal_data = r.split().1;

                    pal.apply_palette_update(pal_data)?;

                    r.seek_relative(block_size as i64 - 4)?;
                }
                _ => {
                    r.seek_relative(-2)?;
                    let frame_header = FrameHeader::new(&mut r)?;

                    if frame_header.is_compressed() {
                        r.seek_relative(6)?;
                        let mut w = Cursor::new(&mut self.buffer);
                        hsq::unhsq(r, &mut w)?;
                        r = Cursor::new(&self.buffer);
                    };

                    let (x, y) = if frame_header.is_full_frame() {
                        (0, 0)
                    } else {
                        (r.read_le_u16()? as i16, r.read_le_u16()? as i16)
                    };

                    let data = r.split().1;

                    blit::Blitter::new(data, framebuffer)
                        .at(x, y)
                        .size(frame_header.width(), frame_header.height())
                        .pal_offset(frame_header.mode())
                        .draw()?;

                    break;
                }
            }
        }

        Ok(())
    }
}
