#![allow(clippy::identity_op)]

use framebuffer::Framebuffer;
use globe_renderer::GlobeRenderer;

fn main() -> Result<(), std::io::Error> {
    const MAP: &[u8] = include_bytes!("../../../assets/MAP.BIN");
    const GLOBDATA: &[u8] = include_bytes!("../../../assets/GLOBDATA.BIN");
    const TABLAT: &[u8] = include_bytes!("../../../assets/TABLAT.BIN");

    let mut globe_renderer = GlobeRenderer::new(GLOBDATA, MAP, TABLAT);

    let mut image_data = vec![0; 320 * 200];
    let mut framebuffer = Framebuffer::new_with_pixel_data(320, 200, image_data.as_mut());

    const PAL: &[u8] = include_bytes!("../../../assets/PAL.BIN");
    for i in 0..256 {
        let r = ((PAL[3 * i + 0] as u32) * 63 / 255) as u8;
        let g = ((PAL[3 * i + 1] as u32) * 63 / 255) as u8;
        let b = ((PAL[3 * i + 2] as u32) * 63 / 255) as u8;
        framebuffer.mut_pal().set(i, (r, g, b));
    }
    framebuffer.clear();

    let rotation = 0;
    let tilt = 0;

    const FRESK: &[u8] = include_bytes!("../../../assets/FRESK.BIN");

    sprite::draw_sprite_from_sprite_sheet(&mut framebuffer, FRESK, 0, 0, 0)?;
    sprite::draw_sprite_from_sprite_sheet(&mut framebuffer, FRESK, 1, 214, 0)?;
    sprite::draw_sprite_from_sprite_sheet(&mut framebuffer, FRESK, 2, 91, 20)?;

    const ICONES: &[u8] = include_bytes!("../../../assets/ICONES.BIN");

    sprite::draw_sprite_from_sprite_sheet(&mut framebuffer, ICONES, 15, 126, 148)?;
    sprite::draw_sprite_from_sprite_sheet(&mut framebuffer, ICONES, 16, 150, 137)?;

    globe_renderer.draw(&mut framebuffer, rotation, tilt);

    framebuffer.write_png_scaled("globe.png")?;

    Ok(())
}
