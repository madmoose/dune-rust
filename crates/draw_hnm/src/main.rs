use std::{fs::read, io::Cursor};

use bytes_ext::ReadBytesExt;
use dune::{Color, Framebuffer, Palette, hnm};

static SKYDN: &[u8] = include_bytes!("../../../assets/SKYDN.BIN");

fn main() {
    let data = read("assets/CRYO.HNM").unwrap();

    let mut pal = Palette::new();
    let mut framebuffer = Framebuffer::new(320, 200);

    apply_sky_palette(3, &mut pal);

    let mut hnm = hnm::HnmDecoder::new(&data, &mut pal).unwrap();

    for frame in 0..hnm.frame_count() {
        hnm.decode_frame(frame, &mut framebuffer, &mut pal).unwrap();
        framebuffer
            .write_ppm(&pal, &format!("CRYO-{frame:04}.ppm"))
            .unwrap();
    }
}

fn apply_sky_palette(sky_palette: usize, pal: &mut Palette) {
    let mut c = Cursor::new(SKYDN);
    let toc_pos = c.read_le_u16().unwrap() as u64;
    c.set_position(toc_pos + (8 + sky_palette.min(32) as u64) * 2);
    let sub_ofs = c.read_le_u16().unwrap() as u64;
    c.set_position(toc_pos + sub_ofs + 6);

    let pal_ofs = 73;
    let pal_cnt = 151;
    for i in 0..pal_cnt {
        let r = c.read_u8().unwrap();
        let g = c.read_u8().unwrap();
        let b = c.read_u8().unwrap();

        pal.set(pal_ofs + i, Color(r, g, b));
    }
}
