use std::{fs::read, io::Cursor, path::PathBuf};

use bytes_ext::ReadBytesExt;
use clap::Parser;
use dune::{Framebuffer, Palette, SpriteSheet, draw_sprite};

#[derive(Parser, Debug)] // requires `derive` feature
struct Args {
    sprite_sheet: PathBuf,
    index: u16,
    sky: Option<usize>,
}

static SKYDN: &[u8] = include_bytes!("../../../assets/SKYDN.BIN");

fn main() {
    let args = Args::parse();

    let Ok(data) = read(&args.sprite_sheet) else {
        eprintln!("Unable to open file `{:?}`", &args.sprite_sheet);
        return;
    };

    let sprite_sheet = match SpriteSheet::from_slice(&data) {
        Ok(sprite_sheet) => sprite_sheet,
        Err(err) => {
            eprintln!("Unable to parse `{:?}`: {}", args.sprite_sheet, err);
            return;
        }
    };

    let Some(sprite) = sprite_sheet.get_sprite(args.index) else {
        eprintln!("Invalid sprite index.");
        return;
    };

    let w = sprite.width();
    let h = sprite.height();

    let mut pal = Palette::new();
    let mut framebuffer = Framebuffer::new(w, h);

    sprite_sheet.apply_palette_update(&mut pal).unwrap();

    if let Some(sky) = args.sky {
        apply_sky_palette(sky, &mut pal);
    }

    draw_sprite(sprite, 0, 0, &mut framebuffer).unwrap();

    let mut path = args.sprite_sheet.file_stem().unwrap_or_default().to_owned();
    path.push(format!("-{:02}", args.index));
    if let Some(sky) = args.sky {
        path.push(format!("-sky_{sky:02}"));
    }
    path.push(".png");

    if let Err(err) = framebuffer.write_png_scaled(&pal, &path) {
        eprintln!("{err:?}");
    };
}

fn apply_sky_palette(palette_index: usize, pal: &mut Palette) {
    let mut c = Cursor::new(SKYDN);

    let toc_pos = c.read_le_u16().unwrap() as u64;
    c.set_position(toc_pos + (8 + palette_index.min(32) as u64) * 2);

    let sub_ofs = c.read_le_u16().unwrap() as u64;
    c.set_position(toc_pos + sub_ofs + 6);

    let pal_ofs = 73;
    let pal_cnt = 151;
    for i in 0..pal_cnt {
        let r = c.read_u8().unwrap();
        let g = c.read_u8().unwrap();
        let b = c.read_u8().unwrap();

        pal.set(pal_ofs + i, (r, g, b));
    }
}
