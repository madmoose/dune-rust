#![allow(clippy::too_many_arguments)]

use std::io::Cursor;

use bytes_ext::{ReadBytesExt, WriteBytesExt};
use framebuffer::{Framebuffer, IndexMap};

pub fn draw(
    index: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    rle: bool,
    flip_x: bool,
    flip_y: bool,
    scale: u8,
    pal_offset: u8,
    data: &[u8],
    frame: &mut Framebuffer,
    index_map: &mut Option<IndexMap>,
) -> std::io::Result<()> {
    let bpp = if pal_offset >= 254 { 8 } else { 4 };
    let pitch = pitch(bpp, width);

    if scale != 0 {
        if bpp != 4 {
            return Ok(());
        }

        if rle {
            let data = unrle(data, pitch, height)?;
            draw_4bpp_scaled(
                &data, frame, x, y, width, height, flip_x, flip_y, scale, pal_offset, index,
                index_map,
            );
        } else {
            draw_4bpp_scaled(
                data, frame, x, y, width, height, flip_x, flip_y, scale, pal_offset, index,
                index_map,
            );
        }
        return Ok(());
    }

    if bpp == 8 {
        if rle {
            let data = unrle(data, pitch, height)?;
            draw_8bpp(
                &data, frame, x, y, width, height, flip_x, flip_y, pal_offset, index, index_map,
            );
        } else {
            draw_8bpp(
                data, frame, x, y, width, height, flip_x, flip_y, pal_offset, index, index_map,
            );
        }
    } else if rle {
        let data = unrle(data, pitch, height)?;
        draw_4bpp(
            &data, frame, x, y, width, height, flip_x, flip_y, pal_offset, index, index_map,
        );
    } else {
        draw_4bpp(
            data, frame, x, y, width, height, flip_x, flip_y, pal_offset, index, index_map,
        );
    }
    Ok(())
}

fn draw_4bpp_scaled(
    src: &[u8],
    frame: &mut Framebuffer,
    dst_x: usize,
    dst_y: usize,
    width: usize,
    height: usize,
    flip_x: bool,
    flip_y: bool,
    scale: u8,
    pal_offset: u8,
    index: usize,
    index_map: &mut Option<IndexMap>,
) {
    let pitch = pitch(4, width);

    let scale_factors: [u16; 8] = [0x100, 0x120, 0x140, 0x160, 0x180, 0x1C0, 0x200, 0x280];
    let scale_factor_fp = scale_factors[scale as usize] as usize;

    let dst_w = (width << 8) / scale_factor_fp;
    let dst_h = (height << 8) / scale_factor_fp;

    let mut src_y_fp = 0;
    for y in 0..dst_h {
        let mut src_x_fp = 0;
        for x in 0..dst_w {
            let src_x = src_x_fp >> 8;
            let src_y = src_y_fp >> 8;

            let mut c = src[src_y * pitch + src_x / 2];
            if src_x & 1 == 0 {
                c &= 0xf;
            } else {
                c >>= 4;
            }

            if c != 0 {
                let x = dst_x + if flip_x { dst_w - x - 1 } else { x };
                let y = dst_y + if flip_y { dst_h - y - 1 } else { y };

                frame.put_pixel(x, y, c + pal_offset);
                if let Some(index_map) = index_map {
                    index_map.set_index(x, y, index)
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
    dst_x: usize,
    dst_y: usize,
    width: usize,
    height: usize,
    flip_x: bool,
    flip_y: bool,
    pal_offset: u8,
    index: usize,
    index_map: &mut Option<IndexMap>,
) {
    let pitch = pitch(4, width);

    for y in 0..height {
        for x in 0..width {
            let mut c = src[y * pitch + x / 2];
            if x & 1 == 0 {
                c &= 0xf;
            } else {
                c >>= 4;
            }

            if c != 0 {
                let x = dst_x + if flip_x { width - x - 1 } else { x };
                let y = dst_y + if flip_y { height - y - 1 } else { y };

                frame.put_pixel(x, y, c + pal_offset);
                if let Some(index_map) = index_map {
                    index_map.set_index(x, y, index);
                }
            }
        }
    }
}

fn draw_8bpp(
    src: &[u8],
    frame: &mut Framebuffer,
    dst_x: usize,
    dst_y: usize,
    width: usize,
    height: usize,
    flip_x: bool,
    flip_y: bool,
    mode: u8,
    index: usize,
    index_map: &mut Option<IndexMap>,
) {
    for y in 0..height {
        for x in 0..width {
            let c = src[y * width + x];
            if mode != 255 || c != 0 {
                let x = dst_x + if flip_x { width - x - 1 } else { x };
                let y = dst_y + if flip_y { height - y - 1 } else { y };
                frame.put_pixel(x, y, c);
                if let Some(index_map) = index_map {
                    index_map.set_index(x, y, index)
                }
            }
        }
    }
}

fn unrle(data: &[u8], pitch: usize, height: usize) -> std::io::Result<Vec<u8>> {
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

fn pitch(bpp: u8, width: usize) -> usize {
    if bpp == 8 {
        width
    } else {
        2 * ((width + 3) / 4)
    }
}
