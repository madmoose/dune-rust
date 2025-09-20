#![allow(clippy::identity_op)]

use dune::{Color, Framebuffer, GlobeRenderer, Palette, SpriteSheet, draw_sprite_from_sheet};

const MAP: &[u8] = include_bytes!("../assets/MAP.BIN");
const GLOBDATA: &[u8] = include_bytes!("../assets/GLOBDATA.BIN");
const TABLAT: &[u8] = include_bytes!("../assets/TABLAT.BIN");

const PAL: &[u8] = include_bytes!("../assets/PAL.BIN");

const FRESK: &[u8] = include_bytes!("../assets/FRESK.BIN");
const ICONES: &[u8] = include_bytes!("../assets/ICONES.BIN");

fn main() -> Result<(), std::io::Error> {
    let mut globe_renderer = GlobeRenderer::new(GLOBDATA, MAP, TABLAT);

    let mut pal = Palette::new();
    let mut framebuffer = Framebuffer::new(320, 200);

    for i in 0..256 {
        let r = ((PAL[3 * i + 0] as u32) * 63 / 255) as u8;
        let g = ((PAL[3 * i + 1] as u32) * 63 / 255) as u8;
        let b = ((PAL[3 * i + 2] as u32) * 63 / 255) as u8;
        pal.set(i, Color(r, g, b));
    }
    framebuffer.clear();

    let rotation = 0;
    let tilt = 0;

    let sprite_sheet = SpriteSheet::from_slice(FRESK)?;
    draw_sprite_from_sheet(&sprite_sheet, 0, 0, 0, &mut framebuffer)?;
    draw_sprite_from_sheet(&sprite_sheet, 1, 214, 0, &mut framebuffer)?;
    draw_sprite_from_sheet(&sprite_sheet, 2, 91, 20, &mut framebuffer)?;

    let sprite_sheet = SpriteSheet::from_slice(ICONES)?;
    let sprite_list = [
        (6, 0, 152),
        (3, 228, 152),
        (13, 22, 161),
        (14, 92, 152),
        (12, 2, 154),
        (12, 317, 154),
        (27, 92, 159),
        (27, 92, 167),
        (27, 92, 175),
        (27, 92, 183),
        (27, 92, 191),
        (41, 266, 171),
        (49, 38, 159),
        (50, 54, 168),
        (51, 38, 183),
        (52, 20, 168),
        (53, 36, 172),
        (15, 126, 148),
        (25, 150, 137),
    ];

    for (id, x, y) in sprite_list {
        draw_sprite_from_sheet(&sprite_sheet, id, x, y, &mut framebuffer)?;
    }

    globe_renderer.draw(&mut framebuffer, rotation, tilt);

    framebuffer.write_png_scaled(&pal, "globe.png")?;

    Ok(())
}
