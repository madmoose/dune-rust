#![feature(cursor_split)]
#![feature(random)]
#![feature(strict_overflow_ops)]
#![allow(clippy::identity_op)]

pub mod attack;
mod color;
mod font;
mod framebuffer;
mod globe_renderer;
mod image;
mod index_map;
mod intro_1;
mod lipsync;
mod palette;
mod point;
mod rect;
mod room_renderer;
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
pub use globe_renderer::GlobeRenderer;
pub use index_map::IndexMap;
pub use lipsync::Lipsync;
pub use palette::Palette;
pub use point::Point;
pub use rect::Rect;
pub use room_renderer::{DrawOptions, Room, RoomRenderer, RoomSheet};
pub use sprite::Sprite;
pub use sprite_blitter::SpriteBlitter;
pub use sprite_sheet::SpriteSheet;

use crate::dat_file::DatFile;

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

pub struct GameState {
    pub dat_file: DatFile,
    // pub hnm_decoder: HnmDecoder;
}
