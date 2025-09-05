#![allow(clippy::too_many_arguments)]

use std::io::Cursor;

use bytes_ext::{ReadBytesExt, WriteBytesExt};

use crate::{Framebuffer, Rect, index_map::IndexMap};

pub struct Blitter<'a> {
    data: &'a [u8],
    framebuffer: &'a mut Framebuffer,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    clip_rect: Option<Rect>,
    flip_x: bool,
    flip_y: bool,
    scale: u8,
    pal_offset: u8,
    rle: bool,
    index: usize,
    index_map: Option<&'a mut IndexMap>,
}

impl<'a> Blitter<'a> {
    pub fn new(data: &'a [u8], framebuffer: &'a mut Framebuffer) -> Self {
        Self {
            data,
            framebuffer,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            clip_rect: None,
            flip_x: false,
            flip_y: false,
            scale: 0,
            rle: false,
            pal_offset: 0,
            index: 0,
            index_map: None,
        }
    }

    pub fn at(mut self, x: i16, y: i16) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn clip_rect(mut self, clip_rect: Option<Rect>) -> Self {
        self.clip_rect = clip_rect;
        self
    }

    pub fn flip_x(mut self, flip_x: bool) -> Self {
        self.flip_x = flip_x;
        self
    }

    pub fn flip_y(mut self, flip_y: bool) -> Self {
        self.flip_y = flip_y;
        self
    }

    pub fn scale(mut self, scale: u8) -> Self {
        self.scale = scale;
        self
    }

    pub fn pal_offset(mut self, pal_offset: u8) -> Self {
        self.pal_offset = pal_offset;
        self
    }

    pub fn rle(mut self, rle: bool) -> Self {
        self.rle = rle;
        self
    }

    pub fn index(mut self, index: usize, index_map: Option<&'a mut IndexMap>) -> Self {
        self.index = index;
        self.index_map = index_map;
        self
    }

    pub fn draw(self) -> std::io::Result<()> {
        draw(
            self.index,
            self.x,
            self.y,
            self.width,
            self.height,
            self.clip_rect,
            self.rle,
            self.flip_x,
            self.flip_y,
            self.scale,
            self.pal_offset,
            self.data,
            self.framebuffer,
            self.index_map,
        )
    }
}

fn draw(
    index: usize,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    clip_rect: Option<Rect>,
    rle: bool,
    flip_x: bool,
    flip_y: bool,
    scale: u8,
    pal_offset: u8,
    data: &[u8],
    frame: &mut Framebuffer,
    index_map: Option<&mut IndexMap>,
) -> std::io::Result<()> {
    assert!(width > 0);
    assert!(width < i16::MAX as u16);
    assert!(height > 0);
    assert!(height < i16::MAX as u16);

    let frame_rect = Rect {
        x0: 0,
        y0: 0,
        x1: frame.w() as i16,
        y1: frame.h() as i16,
    };

    let clip_rect = match clip_rect {
        Some(r) => frame_rect.clip(&r),
        None => frame_rect,
    };

    let bpp = if pal_offset >= 254 { 8 } else { 4 };
    let pitch = pitch(bpp, width);

    if scale != 0 {
        if bpp != 4 {
            return Ok(());
        }

        if rle {
            let data = unrle(data, pitch, height)?;
            draw_4bpp_scaled(
                &data, frame, x, y, width, height, clip_rect, flip_x, flip_y, scale, pal_offset,
                index, index_map,
            );
        } else {
            draw_4bpp_scaled(
                data, frame, x, y, width, height, clip_rect, flip_x, flip_y, scale, pal_offset,
                index, index_map,
            );
        }
        return Ok(());
    }

    if bpp == 8 {
        if rle {
            let data = unrle(data, pitch, height)?;
            draw_8bpp(
                &data, frame, x, y, width, height, clip_rect, flip_x, flip_y, pal_offset, index,
                index_map,
            );
        } else {
            draw_8bpp(
                data, frame, x, y, width, height, clip_rect, flip_x, flip_y, pal_offset, index,
                index_map,
            );
        }
    } else if rle {
        let data = unrle(data, pitch, height)?;
        draw_4bpp(
            &data, frame, x, y, width, height, clip_rect, flip_x, flip_y, pal_offset, index,
            index_map,
        );
    } else {
        draw_4bpp(
            data, frame, x, y, width, height, clip_rect, flip_x, flip_y, pal_offset, index,
            index_map,
        );
    }
    Ok(())
}

fn draw_4bpp_scaled(
    src: &[u8],
    frame: &mut Framebuffer,
    dst_x: i16,
    dst_y: i16,
    width: u16,
    height: u16,
    clip_rect: Rect,
    flip_x: bool,
    flip_y: bool,
    scale: u8,
    pal_offset: u8,
    index: usize,
    index_map: Option<&mut IndexMap>,
) {
    let mut index_map = index_map;
    let pitch = pitch(4, width);

    let scale_factors: [u16; 8] = [0x100, 0x120, 0x140, 0x160, 0x180, 0x1C0, 0x200, 0x280];
    let scale_factor_fp = scale_factors[scale as usize];

    let dst_w = (width << 8) / scale_factor_fp;
    let dst_h = (height << 8) / scale_factor_fp;

    let mut src_y_fp = 0;
    for y in 0..dst_h {
        let mut src_x_fp = 0;
        for x in 0..dst_w {
            let src_x = src_x_fp >> 8;
            let src_y = src_y_fp >> 8;

            let src_offset = src_y * pitch + src_x / 2;
            let mut c = src[src_offset as usize];
            if src_x & 1 == 0 {
                c &= 0xf;
            } else {
                c >>= 4;
            }

            if c != 0 {
                let x = dst_x.saturating_add_unsigned(if flip_x { dst_w - x - 1 } else { x });
                let y = dst_y.saturating_add_unsigned(if flip_y { dst_h - y - 1 } else { y });

                if !clip_rect.in_rect(x, y) {
                    continue;
                }

                let x = x as u16;
                let y = y as u16;
                frame.set(x, y, c + pal_offset);
                if let Some(index_map) = index_map.as_deref_mut() {
                    index_map.set_index(x, y, index);
                }
            }
            src_x_fp += scale_factor_fp;
        }
        src_y_fp += scale_factor_fp;
    }
}

fn draw_4bpp(
    src: &[u8],
    frame: &mut Framebuffer,
    dst_x: i16,
    dst_y: i16,
    width: u16,
    height: u16,
    clip_rect: Rect,
    flip_x: bool,
    flip_y: bool,
    pal_offset: u8,
    index: usize,
    index_map: Option<&mut IndexMap>,
) {
    let mut index_map = index_map;
    let pitch = pitch(4, width);

    for y in 0..height {
        for x in 0..width {
            let src_offset = y * pitch + x / 2;
            let mut c = src[src_offset as usize];
            if x & 1 == 0 {
                c &= 0xf;
            } else {
                c >>= 4;
            }

            if c != 0 {
                let x = dst_x.saturating_add_unsigned(if flip_x { width - x - 1 } else { x });
                let y = dst_y.saturating_add_unsigned(if flip_y { height - y - 1 } else { y });

                if !clip_rect.in_rect(x, y) {
                    continue;
                }

                let x = x as u16;
                let y = y as u16;
                frame.set(x, y, c + pal_offset);
                if let Some(index_map) = index_map.as_deref_mut() {
                    index_map.set_index(x, y, index);
                }
            }
        }
    }
}

fn draw_8bpp(
    src: &[u8],
    frame: &mut Framebuffer,
    dst_x: i16,
    dst_y: i16,
    width: u16,
    height: u16,
    clip_rect: Rect,
    flip_x: bool,
    flip_y: bool,
    mode: u8,
    index: usize,
    index_map: Option<&mut IndexMap>,
) {
    let mut index_map = index_map;

    for y in 0..height {
        for x in 0..width {
            let src_offset = y * width + x;
            let c = src[src_offset as usize];
            if mode != 255 || c != 0 {
                let x = dst_x.saturating_add_unsigned(if flip_x { width - x - 1 } else { x });
                let y = dst_y.saturating_add_unsigned(if flip_y { height - y - 1 } else { y });

                if !clip_rect.in_rect(x, y) {
                    continue;
                }

                let x = x as u16;
                let y = y as u16;
                frame.set(x, y, c);
                if let Some(index_map) = index_map.as_deref_mut() {
                    index_map.set_index(x, y, index);
                }
            }
        }
    }
}

fn unrle(data: &[u8], pitch: u16, height: u16) -> std::io::Result<Vec<u8>> {
    let pitch = pitch as usize;
    let height = height as usize;

    let mut buf = vec![0u8; height * pitch];

    let mut rle_src = Cursor::new(data);
    let mut rle_dst = Cursor::new(&mut buf);

    for _ in 0..height {
        let mut x = 0;
        while x < pitch {
            let count;
            let cmd = rle_src.read_u8()?;
            if cmd & 0x80 != 0 {
                count = 257 - (cmd as usize);
                let value = rle_src.read_u8()?;
                for _ in 0..count {
                    rle_dst.write_u8(value)?;
                }
            } else {
                count = (cmd as usize) + 1;
                for _ in 0..count {
                    let value = rle_src.read_u8()?;
                    rle_dst.write_u8(value)?;
                }
            }

            x += count;
        }
    }

    Ok(buf)
}

fn pitch(bpp: u8, width: u16) -> u16 {
    assert!(width > 0);
    if bpp == 8 {
        width
    } else {
        2 * width.div_ceil(4)
    }
}
