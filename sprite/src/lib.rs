#![allow(clippy::identity_op)]
#![feature(cursor_split)]

use std::io::{self, Cursor};

mod sprite;
mod sprite_sheet;

pub use sprite::Sprite;
pub use sprite_sheet::SpriteSheet;

use bytes_ext::ReadBytesExt;
use framebuffer::Framebuffer;

pub fn draw_sprite_from_sprite_sheet(
    dst: &mut Framebuffer,
    src: &[u8],
    sprite_index: u16,
    x: usize,
    y: usize,
) -> std::io::Result<()> {
    let mut src = Cursor::new(src);

    let toc_position = src.read_le_u16()? as u64;
    src.set_position(toc_position);

    let first_resource_offset = src.read_le_u16()?;
    let sub_resource_count = first_resource_offset / 2;

    if sub_resource_count == 0 || sub_resource_count > 1000 {
        panic!("Not a sprite sheet");
    }

    if sprite_index >= sub_resource_count {
        panic!("Invalid sprite index");
    }

    src.set_position(toc_position + 2 * sprite_index as u64);
    let sprite_offset = src.read_le_u16()? as u64;

    src.set_position(toc_position + sprite_offset);

    draw_sprite(dst, src.split().1, x, y)?;

    Ok(())
}

fn draw_sprite(dst: &mut Framebuffer, src: &[u8], x: usize, y: usize) -> std::io::Result<()> {
    let mut src = Cursor::new(src);

    let w0 = src.read_le_u16()?;
    let w1 = src.read_le_u16()?;

    let flags = ((w0 & 0xff00) >> 8) as u8;
    let width = (w0 & 0x7fff) as usize;
    let height = (w1 & 0x00ff) as usize;
    let pal_offset = ((w1 & 0xff00) >> 8) as u8;

    if !(1..=320).contains(&width) || !(1..=200).contains(&height) {
        return Ok(());
    }

    let is_rle_compressed = flags & 0x80 != 0;

    if pal_offset < 254 {
        if !is_rle_compressed {
            draw_4bpp(dst, src.split().1, x, y, width, height, pal_offset)?;
        } else {
            draw_4bpp_rle(dst, src.split().1, x, y, width, height, pal_offset)?;
        }
    } else if !is_rle_compressed {
        draw_8bpp(dst, src.split().1, x, y, width, height, pal_offset)?;
    } else {
        draw_8bpp_rle(dst, src.split().1, x, y, width, height, pal_offset)?;
    }

    Ok(())
}

fn draw_4bpp(
    dst: &mut Framebuffer,
    src: &[u8],
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    mode: u8,
) -> io::Result<()> {
    let mut src = Cursor::new(src);
    for y in 0..h {
        let mut line_remain = 4 * ((w + 3) / 4);
        let mut x = 0;
        while line_remain > 0 {
            let value = src.read_u8()?;
            let p1 = value & 0x0f;
            let p2 = value >> 4;

            if p1 != 0 && x < w {
                dst.put_pixel(x + x0, y + y0, p1 + mode);
            }
            x += 1;

            if p2 != 0 && x < w {
                dst.put_pixel(x + x0, y + y0, p2 + mode);
            }
            x += 1;

            line_remain -= 2;
        }
    }

    Ok(())
}

fn draw_4bpp_rle(
    dst: &mut Framebuffer,
    src: &[u8],
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    mode: u8,
) -> io::Result<()> {
    let mut src = Cursor::new(src);

    for y in 0..h {
        let mut line_remain = 4 * ((w + 3) / 4);
        let mut x = 0;
        while line_remain > 0 {
            let cmd = src.read_u8()?;
            if cmd & 0x80 != 0 {
                let count = 257 - (cmd as u16);
                let value = src.read_u8()?;

                let p1 = value & 0x0f;
                let p2 = value >> 4;
                for _ in 0..count {
                    if p1 != 0 {
                        dst.put_pixel(x + x0, y + y0, p1 + mode);
                    }
                    x += 1;
                    if p2 != 0 {
                        dst.put_pixel(x + x0, y + y0, p2 + mode);
                    }
                    x += 1;
                }
                line_remain -= 2 * (count as usize);
            } else {
                let count = (cmd + 1) as u16;
                for _ in 0..count {
                    let value = src.read_u8()?;

                    let p1 = value & 0x0f;
                    let p2 = value >> 4;

                    if p1 != 0 {
                        dst.put_pixel(x + x0, y + y0, p1 + mode);
                    }
                    x += 1;

                    if p2 != 0 {
                        dst.put_pixel(x + x0, y + y0, p2 + mode);
                    }
                    x += 1;
                }
                line_remain -= 2 * (count as usize);
            }
        }
    }

    Ok(())
}

fn draw_8bpp(
    dst: &mut Framebuffer,
    src: &[u8],
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    mode: u8,
) -> io::Result<()> {
    let mut src = Cursor::new(src);

    for y in 0..h {
        for x in 0..w {
            let value = src.read_u8()?;
            if mode != 255 && value != 0 {
                dst.put_pixel(x + x0, y + y0, value);
            }
        }
    }

    Ok(())
}

fn draw_8bpp_rle(
    dst: &mut Framebuffer,
    src: &[u8],
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    mode: u8,
) -> io::Result<()> {
    let mut src = Cursor::new(src);

    for y in 0..h {
        let mut x = 0;

        while x < w {
            let cmd = src.read_u8()?;
            if cmd & 0x80 != 0 {
                let count = 257 - (cmd as u16);
                let value = src.read_u8()?;
                for _ in 0..count {
                    if (mode != 255 || value != 0) && x < w {
                        dst.put_pixel(x + x0, y + y0, value);
                    }
                    x += 1;
                }
            } else {
                let count = (cmd + 1) as u16;
                for _ in 0..count {
                    let value = src.read_u8()?;
                    if (mode != 255 || value != 0) && x < w {
                        dst.put_pixel(x + x0, y + y0, value);
                    }
                    x += 1;
                }
            }
        }
    }

    Ok(())
}
