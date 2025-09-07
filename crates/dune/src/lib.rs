#![feature(cursor_split)]
#![allow(clippy::identity_op)]

pub mod attack;
mod color;
mod font;
mod framebuffer;
mod image;
mod index_map;
mod palette;
mod point;
mod rect;
mod sprite;
mod sprite_blitter;
mod sprite_sheet;

pub mod blit;
pub mod dat_file;
pub mod hnm;
pub mod hsq;

pub use color::Color;
pub use font::{Font, TextAlign, TextContext, TextSize, TextStyle, draw_text};
pub use framebuffer::Framebuffer;
pub use index_map::IndexMap;
pub use palette::Palette;
pub use point::Point;
pub use rect::Rect;
pub use sprite::Sprite;
pub use sprite_blitter::SpriteBlitter;
pub use sprite_sheet::SpriteSheet;

pub fn sprite_blitter<'a>(
    sprite: &'a Sprite,
    framebuffer: &'a mut Framebuffer,
) -> SpriteBlitter<'a> {
    SpriteBlitter::new(sprite, framebuffer)
}

pub fn draw_sprite(
    sprite: &Sprite,
    x: i16,
    y: i16,
    framebuffer: &mut Framebuffer,
) -> std::io::Result<()> {
    blit::Blitter::new(sprite.data(), framebuffer)
        .at(x, y)
        .size(sprite.width(), sprite.height())
        .pal_offset(sprite.pal_offset())
        .rle(sprite.rle())
        .draw()
}

pub fn draw_sprite_from_sheet(
    sheet: &SpriteSheet,
    sprite_id: u16,
    x: i16,
    y: i16,
    framebuffer: &mut Framebuffer,
) -> std::io::Result<()> {
    if let Some(sprite) = sheet.get_sprite(sprite_id) {
        draw_sprite(sprite, x, y, framebuffer)
    } else {
        Ok(())
    }
}
