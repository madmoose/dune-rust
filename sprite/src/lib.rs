#![allow(clippy::identity_op)]
#![feature(cursor_split)]
#![feature(debug_closure_helpers)]

use std::io::Cursor;

mod draw_builder;
mod sprite;
mod sprite_sheet;

use bytes_ext::ReadBytesExt;
pub use draw_builder::DrawBuilder;
use framebuffer::Framebuffer;
pub use sprite::Sprite;
pub use sprite_sheet::SpriteSheet;

pub fn draw_sprite_from_sprite_sheet(
    framebuffer: &mut Framebuffer,
    src: &[u8],
    sprite_index: u16,
    x: i16,
    y: i16,
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

    if let Some(sprite) = Sprite::from_slice(src.split().1) {
        sprite.draw(x, y, framebuffer)?;
    }

    Ok(())
}
