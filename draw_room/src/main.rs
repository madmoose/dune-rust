use framebuffer::{Framebuffer, Palette};
use room_renderer::{DrawOptions, RoomRenderer, RoomSheet};
use sprite::SpriteSheet;

static ROOMS_SHEET: &[u8] = include_bytes!("../../assets/PALACE.SAL");
static SPRITE_SHEET: &[u8] = include_bytes!("../../assets/POR.BIN");
static SKYDN: &[u8] = include_bytes!("../../assets/SKYDN.BIN");

fn main() {
    let room_sheet = RoomSheet::new(ROOMS_SHEET).unwrap();
    let room = room_sheet.get_room(1).unwrap();
    let sprite_sheet = SpriteSheet::from_slice(SPRITE_SHEET).unwrap();

    let mut pal = Palette::new();
    let mut framebuffer = Framebuffer::new(320, 200);

    let mut room_renderer = RoomRenderer::new();

    room_renderer.draw_sky(SKYDN, 8, &mut pal);

    sprite_sheet.apply_palette_update(&mut pal).unwrap();

    room_renderer.set_room(room.to_owned());
    room_renderer.set_sprite_sheet(sprite_sheet);

    room_renderer
        .draw(
            &DrawOptions {
                draw_sprites: true,
                draw_polygons: true,
                draw_lines: true,
            },
            &mut framebuffer,
            &mut None,
        )
        .unwrap();

    framebuffer.write_png_scaled(&pal, "room.png").unwrap();
}
