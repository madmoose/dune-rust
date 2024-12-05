use framebuffer::Framebuffer;
use room_renderer::{DrawOptions, RoomRenderer, RoomSheet};
use sprite::SpriteSheet;

static ROOMS_SHEET: &[u8] = include_bytes!("../../assets/PALACE.SAL");
static SPRITE_SHEET: &[u8] = include_bytes!("../../assets/POR.BIN");
static SKYDN: &[u8] = include_bytes!("../../assets/SKYDN.BIN");

fn main() {
    let room_sheet = RoomSheet::new(ROOMS_SHEET).unwrap();
    let room = room_sheet.get_room(1).unwrap();
    let sprite_sheet = SpriteSheet::new(SPRITE_SHEET).unwrap();

    let mut buffer = vec![0; 320 * 200];
    let mut framebuffer = Framebuffer::new_with_pixel_data(320, 200, &mut buffer);

    let mut room_renderer = RoomRenderer::new();

    room_renderer.draw_sky(SKYDN, 8, &mut framebuffer);

    sprite_sheet
        .apply_palette_update(framebuffer.mut_pal())
        .unwrap();

    room_renderer.set_room(room.to_owned());
    room_renderer.set_sprite_sheet(sprite_sheet);

    room_renderer.draw(
        &DrawOptions {
            draw_sprites: true,
            draw_polygons: true,
            draw_lines: true,
        },
        &mut framebuffer,
        &mut None,
    );

    framebuffer.write_png_scaled("room.png").unwrap();
}
