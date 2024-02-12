use framebuffer::Framebuffer;
use globe_renderer::GlobeRenderer;

fn main() -> Result<(), std::io::Error> {
    const MAP: &[u8] = include_bytes!("../../../assets/MAP.BIN");
    const GLOBDATA: &[u8] = include_bytes!("../../../assets/GLOBDATA.BIN");
    const TABLAT: &[u8] = include_bytes!("../../../assets/TABLAT.BIN");

    let mut globe_renderer = GlobeRenderer::new(GLOBDATA, MAP, TABLAT);

    const PAL: &[u8] = include_bytes!("../../../assets/PAL.BIN");
    let mut image_data = vec![0; 4 * 320 * 200];
    let mut fb =
        Framebuffer::new_with_pixel_data(320, 200, image_data.as_mut(), PAL.try_into().unwrap());

    fb.clear();

    let rotation = 0;
    let tilt = 0;

    const FRESK: &[u8] = include_bytes!("../../../assets/FRESK.BIN");

    sprite::draw_sprite_from_sprite_sheet(&mut fb, FRESK, 0, 0, 0)?;
    sprite::draw_sprite_from_sprite_sheet(&mut fb, FRESK, 1, 214, 0)?;
    sprite::draw_sprite_from_sprite_sheet(&mut fb, FRESK, 2, 91, 20)?;

    const ICONES: &[u8] = include_bytes!("../../../assets/ICONES.BIN");

    sprite::draw_sprite_from_sprite_sheet(&mut fb, ICONES, 15, 126, 148)?;
    sprite::draw_sprite_from_sprite_sheet(&mut fb, ICONES, 16, 150, 137)?;

    globe_renderer.draw(&mut fb, rotation, tilt);

    fb.write_ppm("out.ppm")?;

    Ok(())
}
