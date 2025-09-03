use std::{io::Cursor, path::PathBuf};

use bytes_ext::ReadBytesExt;
use clap::Parser;
use dune::{dat_file::DatFile, hsq};
use framebuffer::{Framebuffer, Palette};
use lipsync::Lipsync;
use sprite::SpriteSheet;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'd')]
    dat_file: Option<PathBuf>,

    #[arg(required = true)]
    resources: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let mut dat_file = args
        .dat_file
        .as_deref()
        .map(|path| DatFile::open(path).expect("read dat-file"));

    for filename in &args.resources {
        let path = PathBuf::from(filename);

        let data = if let Some(dat_file) = dat_file.as_mut() {
            dat_file.read(filename).expect("file")
        } else {
            let is_compressed = path
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("hsq") || ext.eq_ignore_ascii_case("sqz"))
                .unwrap_or_default();

            if is_compressed {
                let compressed_data = std::fs::read(&path)
                    .unwrap_or_else(|_| panic!("Unable to read file `{path:?}`"));
                let mut r = Cursor::new(&compressed_data);

                let header = hsq::Header::from_reader(&mut r).unwrap();

                let mut data = Vec::with_capacity(header.uncompressed_size() as usize);
                let mut w = Cursor::new(&mut data);

                hsq::unhsq(&mut r, &mut w).expect("Unable to decompress file");
                data
            } else {
                std::fs::read(&path).expect("Unable to read file")
            }
        };

        let output_file_stem = path
            .file_stem()
            .unwrap_or_default()
            .to_owned()
            .to_string_lossy()
            .to_ascii_lowercase();

        let sprite_sheet = SpriteSheet::from_slice(&data).unwrap();
        let last_resource_id = sprite_sheet.resource_count() - 1;
        let lipsync_data = sprite_sheet.get_resource(last_resource_id).unwrap();
        let lipsync = Lipsync::from_bytes(lipsync_data);

        let mut pal = Palette::new();
        sprite_sheet.apply_palette_update(&mut pal).unwrap();

        let w = lipsync.rect.3.next_multiple_of(2) as usize;
        let h = lipsync.rect.2.next_multiple_of(2) as usize;
        let mut framebuffer = Framebuffer::new(h, w);

        lipsync.display();

        if true {
            let voc_data: [u8; 93] = [
                0x0C, 0x0C, 0x0C, 0x0C, 0x0B, 0x0B, 0x0B, 0x0C, 0x0C, 0x07, 0x01, 0x01, 0x02, 0x03,
                0x03, 0x09, 0x09, 0x03, 0x03, 0x03, 0x06, 0x06, 0x06, 0x0A, 0x0A, 0x0A, 0x0A, 0x0B,
                0x0C, 0x0C, 0x0B, 0x09, 0x09, 0x03, 0x03, 0x02, 0x07, 0x0B, 0x0B, 0x09, 0x08, 0x03,
                0x02, 0x03, 0x0B, 0x0C, 0x0B, 0x0B, 0x0A, 0x0A, 0x0A, 0x0A, 0x06, 0x06, 0x06, 0x06,
                0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x03, 0x03, 0x03, 0x09,
                0x08, 0x02, 0x06, 0x06, 0x06, 0x0B, 0x0C, 0x0B, 0x08, 0x09, 0x09, 0x03, 0x03, 0x02,
                0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0xFF,
            ];

            let mut r = Cursor::new(&voc_data[..]);
            let mut index = 0;
            let last_animation_idx = lipsync.animation_count() - 1;

            loop {
                let v: u16 = r.read_u8().unwrap() as u16;
                if v == 0xff {
                    break;
                }

                framebuffer.clear();

                lipsync.draw_animation_frame(
                    &mut framebuffer,
                    &sprite_sheet,
                    last_animation_idx,
                    v as usize + 1,
                );

                framebuffer
                    .write_ppm_scaled(
                        &pal,
                        &format!("out/{output_file_stem}-voc-{index:02}.ppm"),
                        5,
                        6,
                    )
                    .unwrap();
                index += 1;
            }
        } else {
            for i in 0..lipsync.animation_count() {
                for j in 0..lipsync.animation_frame_count(i) {
                    framebuffer.clear();

                    lipsync.draw_animation_frame(&mut framebuffer, &sprite_sheet, i, j);

                    framebuffer
                        .write_ppm_scaled(
                            &pal,
                            &format!("out/{output_file_stem}-animation-{i:02}-frame-{j:02}.ppm"),
                            5,
                            6,
                        )
                        .unwrap();
                }
            }
        }
    }
}
