use std::io::Cursor;

use bytes_ext::ReadBytesExt;
use framebuffer::Framebuffer;

const MAX_TILT: usize = 99;

#[derive(Copy, Clone, Debug, Default)]
struct RotationEntry {
    map_row_start: i16,
    map_row_len: u16,
    fp: u32,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum GlobeSection {
    FarNorth,
    NearNorth,
    NearSouth,
    FarSouth,
}

#[derive(Copy, Clone, Debug)]
struct GlobeSectionLatitude {
    pub section: GlobeSection,
    pub latitude: u8,
}

impl GlobeSectionLatitude {
    fn new(section: GlobeSection, latitude: u8) -> GlobeSectionLatitude {
        GlobeSectionLatitude { section, latitude }
    }
}

impl Default for GlobeSectionLatitude {
    fn default() -> Self {
        GlobeSectionLatitude::new(GlobeSection::FarNorth, 0)
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Half {
    Upper,
    Lower,
}

pub struct GlobeRenderer {
    globdata: Vec<u8>,
    map: Vec<u8>,
    rotation_lookup_table: [RotationEntry; MAX_TILT],
    tilt_lookup_table: [GlobeSectionLatitude; 4 * MAX_TILT - 4],
}

impl GlobeRenderer {
    pub fn new(globdata: &[u8], map: &[u8], tablat: &[u8]) -> GlobeRenderer {
        let mut r = GlobeRenderer {
            globdata: globdata.to_vec(),
            map: map.to_vec(),
            rotation_lookup_table: [RotationEntry::default(); MAX_TILT],
            tilt_lookup_table: [GlobeSectionLatitude::default(); 4 * MAX_TILT - 4],
        };

        let mut tilt_lookup_table = Vec::with_capacity(r.tilt_lookup_table.len());
        for i in 1..=98 {
            tilt_lookup_table.push(GlobeSectionLatitude::new(GlobeSection::FarSouth, i));
        }
        for i in (0..=98).rev() {
            tilt_lookup_table.push(GlobeSectionLatitude::new(GlobeSection::NearSouth, i));
        }
        for i in 1..=98 {
            tilt_lookup_table.push(GlobeSectionLatitude::new(GlobeSection::NearNorth, i));
        }
        for i in (2..=98).rev() {
            tilt_lookup_table.push(GlobeSectionLatitude::new(GlobeSection::FarNorth, i));
        }
        r.tilt_lookup_table = tilt_lookup_table.try_into().unwrap();

        for (i, e) in r.rotation_lookup_table.iter_mut().enumerate() {
            let offset = 8 * i;
            e.map_row_start = i16::from_be_bytes(tablat[offset..offset + 2].try_into().unwrap());
            e.map_row_len = u16::from_be_bytes(tablat[offset + 2..offset + 4].try_into().unwrap());
        }

        r.precalculate_globe_rotation_lookup_table(0);
        r
    }

    fn globdata_table_2(&self, x: usize, latitude: usize) -> u8 {
        assert!(x < 64);
        assert!(latitude < 100);
        self.globdata[3290 + x * 200 + latitude]
    }

    fn globdata_table_3(&self, x: usize, latitude: usize) -> i16 {
        assert!(x < 64);
        assert!(latitude < 100);
        self.globdata[3290 + x * 200 + latitude + 100] as i8 as i16
    }

    fn precalculate_globe_rotation_lookup_table(&mut self, rotation: u16) {
        let mut dxax: u32 = 398 * rotation as u32;
        dxax &= !0xffff;

        self.rotation_lookup_table[0].fp = dxax;
        dxax += 0x8000;

        let bx = dxax / 398;
        for i in 1..self.rotation_lookup_table.len() {
            let dxax = 2 * bx * self.rotation_lookup_table[i].map_row_len as u32;

            self.rotation_lookup_table[i].fp = dxax;
        }
    }

    fn map_color(&self, offset: i16) -> u8 {
        let map_value = self.map[(0x62FCi32 + offset as i32) as usize];
        let flags = (map_value >> 4) & 3;
        let mut color = map_value & 0x0f;

        if flags == 0x10 && color < 8 {
            color += 12;
        }

        color + 0x10
    }

    fn draw_half(&self, fb: &mut Framebuffer, half: Half, tilt: i16) {
        let center_x = 160 - 1;
        let center_y = 80 - 1;

        let mut y: i32 = 0;
        let mut globdata_reader = Cursor::new(&self.globdata);

        loop {
            let n = globdata_reader.read_i8().unwrap();
            assert!(n < 0);

            let line_len = !n as i32;
            if line_len == 0 {
                break;
            }

            for x in 0..line_len {
                let n = match half {
                    Half::Upper => globdata_reader.read_i8().unwrap(),
                    Half::Lower => -globdata_reader.read_i8().unwrap(),
                };

                let section_latitude = self.tilt_lookup_table[(n as i16 + 196 + tilt) as usize];

                let bx_ = self.globdata_table_2(x as usize, section_latitude.latitude as usize);
                let mut ax = self.globdata_table_3(x as usize, section_latitude.latitude as usize);

                let bp = (bx_ / 2) as usize;
                let mut bx = self.rotation_lookup_table[bp].map_row_start;
                let mut cx = self.rotation_lookup_table[bp].map_row_len as i16;
                let mut dx = (self.rotation_lookup_table[bp].fp >> 16) as i16;

                match section_latitude.section {
                    GlobeSection::FarNorth => {
                        ax = cx - ax;
                        bx = -bx;
                    }
                    GlobeSection::NearNorth => {
                        bx = -bx;
                    }
                    GlobeSection::NearSouth => {}
                    GlobeSection::FarSouth => {
                        ax = cx - ax;
                    }
                };

                cx *= 2;
                let mut bp = dx - ax;
                if bp < 0 {
                    bp += cx;
                }
                bp += bx;
                dx += ax;

                let color = self.map_color(bp);
                let py = match half {
                    Half::Upper => center_y - y,
                    Half::Lower => center_y + y,
                };
                fb.put_pixel((center_x - x) as usize, py as usize, color);

                let mut bp = dx - cx;
                if bp < 0 {
                    bp += cx;
                }
                bp += bx;

                let color = self.map_color(bp);
                let py = match half {
                    Half::Upper => center_y - y,
                    Half::Lower => center_y + y,
                };
                fb.put_pixel((center_x + x + 1) as usize, py as usize, color);
            }

            y += 1;
        }
    }

    pub fn draw(&mut self, fb: &mut Framebuffer, rotation: u16, tilt: i16) {
        let tilt = tilt.clamp(-96, 96);
        self.precalculate_globe_rotation_lookup_table(rotation);
        self.draw_half(fb, Half::Upper, tilt);
        self.draw_half(fb, Half::Lower, tilt);
    }
}
