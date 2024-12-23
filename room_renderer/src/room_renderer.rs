use std::{io::Cursor, mem::swap};

use bytes_ext::ReadBytesExt;
use framebuffer::{Framebuffer, IndexMap};
use sprite::SpriteSheet;

use crate::{
    room::{Part, Polygon},
    Room,
};

pub struct RoomRenderer {
    room: Option<Room>,
    sprite_sheet: Option<SpriteSheet>,
    y_offset: usize,
}

pub struct DrawOptions {
    pub draw_sprites: bool,
    pub draw_polygons: bool,
    pub draw_lines: bool,
}

impl Default for DrawOptions {
    fn default() -> Self {
        Self {
            draw_sprites: true,
            draw_polygons: true,
            draw_lines: true,
        }
    }
}

impl RoomRenderer {
    pub fn new() -> Self {
        Self {
            room: None,
            sprite_sheet: None,
            y_offset: 0,
        }
    }

    pub fn set_room(&mut self, room: Room) {
        self.room = Some(room);
    }

    pub fn set_sprite_sheet(&mut self, sprite_sheet: SpriteSheet) {
        self.sprite_sheet = Some(sprite_sheet);
    }

    pub fn get_sprite_sheet(&mut self) -> Option<&SpriteSheet> {
        self.sprite_sheet.as_ref()
    }

    pub fn draw_and_write_ppm_parts(
        &self,
        room: &Room,
        sprite_sheet: &SpriteSheet,
        options: &DrawOptions,
        frame: &mut Framebuffer,
    ) {
        for (i, part) in room.parts().iter().enumerate() {
            if self.draw_part(i, part, sprite_sheet, options, frame, &mut None) {
                let filename = format!("room-part-{i:02}-pal.h");
                frame.write_pal_as_header(&filename).unwrap();

                let filename = format!("room-part-{i:02}.ppm");
                frame.write_ppm_scaled(&filename, 5, 6).unwrap();
            }
        }
    }

    pub fn draw_sky(&self, sky_asset: &[u8], sky_palette: usize, frame: &mut Framebuffer) {
        let mut c = Cursor::new(sky_asset);
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

            frame.mut_pal().set(pal_ofs + i, (r, g, b));
        }

        // let sky_sprite_sheet = SpriteSheet::new(sky_asset).unwrap();

        // for sprite_id in 0..4 {
        //     let sprite = sky_sprite_sheet.get_sprite(sprite_id).unwrap();
        //     for col in 0..8 {
        //         let y = 20 * sprite_id;
        //         let x = 40 * col;
        //         sprite
        //             .draw(sprite_id, x, y, false, false, 0, 0, frame, &mut None)
        //             .unwrap();
        //     }
        // }
    }

    pub fn draw(
        &self,
        options: &DrawOptions,
        frame: &mut Framebuffer,
        index_map: &mut Option<IndexMap>,
    ) -> usize {
        let Some(room) = &self.room else {
            return 0;
        };
        let Some(sprite_sheet) = &self.sprite_sheet else {
            return 0;
        };

        let mut parts_drawn = 0;
        for (i, part) in room.parts().iter().enumerate() {
            if self.draw_part(i, part, sprite_sheet, options, frame, index_map) {
                parts_drawn += 1;
            }
        }

        parts_drawn
    }

    fn draw_part(
        &self,
        index: usize,
        part: &Part,
        sprite_sheet: &SpriteSheet,
        options: &DrawOptions,
        frame: &mut Framebuffer<'_>,
        index_map: &mut Option<IndexMap>,
    ) -> bool {
        match part {
            Part::Sprite(sprite_part) => {
                if !options.draw_sprites {
                    return false;
                }
                let Some(sprite) = sprite_sheet.get_sprite(sprite_part.id as usize) else {
                    return false;
                };

                sprite
                    .draw(
                        index,
                        sprite_part.x as usize,
                        sprite_part.y as usize + self.y_offset,
                        sprite_part.flip_x,
                        sprite_part.flip_y,
                        sprite_part.scale,
                        sprite_part.pal_offset,
                        frame,
                        index_map,
                    )
                    .unwrap();
                true
            }
            Part::Character(_) => false,
            Part::Polygon(polygon_part) => {
                if !options.draw_polygons {
                    return false;
                }

                self.draw_polygon(index, polygon_part, frame, index_map);
                true
            }
            Part::Line(line_part) => {
                if !options.draw_lines {
                    return false;
                }
                self.draw_line(
                    index,
                    line_part.p0,
                    line_part.p1,
                    line_part.color,
                    line_part.dither,
                    frame,
                    index_map,
                );
                true
            }
        }
    }

    fn draw_line(
        &self,
        index: usize,
        p0: (u16, u16),
        p1: (u16, u16),
        color: u8,
        dither: u16,
        frame: &mut Framebuffer,
        index_map: &mut Option<IndexMap>,
    ) {
        let mut dither = dither;

        bresenham_line(p0, p1, |x, y| {
            dither = dither.rotate_left(1);
            if dither & 1 != 0 {
                let y = y + self.y_offset;
                frame.put_pixel(x, y, color);
                if let Some(m) = index_map.as_mut() {
                    m.set_index(x, y, index)
                }
            }
        });
    }

    fn draw_polygon(
        &self,
        index: usize,
        polygon: &Polygon,
        frame: &mut Framebuffer,
        index_map: &mut Option<IndexMap>,
    ) {
        let mut right_side = [0u16; 200];
        let mut left_side = [0u16; 200];

        let mut xi = 0;
        let start_p = polygon.right_vertices[0];

        // Part 1
        let mut last_p = polygon.right_vertices[0];
        polygon.right_vertices.iter().skip(1).for_each(|&p| {
            draw_edge(last_p, p, &mut right_side, &mut xi);
            last_p = p;
        });
        let final_p = last_p;

        // Part 2
        xi = 0;
        let mut last_p = polygon.right_vertices[0];
        polygon.left_vertices.iter().for_each(|&p| {
            draw_edge(last_p, p, &mut left_side, &mut xi);
            last_p = p;
        });

        draw_edge(last_p, final_p, &mut left_side, &mut xi);

        let mut noise_generator = polygon.noise.clone();
        let mut line_color = (polygon.color as u16) << 8;

        for y in 0..final_p.1 - start_p.1 {
            let y = y as usize;
            let mut x0 = left_side[y];
            let mut x1 = right_side[y];
            if x0 > x1 {
                swap(&mut x0, &mut x1);
            }

            let mut color = line_color;
            for x in x0..=x1 {
                let rand = noise_generator.rand() & 3;

                let x = if !polygon.reverse_gradient {
                    x as usize
                } else {
                    (x0 + (x1 - x)) as usize
                };

                let y = y + start_p.1 as usize + self.y_offset;

                frame.put_pixel(x, y, (rand + (color >> 8) - 1) as u8);
                if let Some(index_map) = index_map.as_mut() {
                    index_map.set_index(x, y, index)
                }
                color = color.strict_add_signed(polygon.h_gradient);
            }
            line_color = line_color.strict_add_signed(polygon.v_gradient);
        }
    }
}

impl Default for RoomRenderer {
    fn default() -> Self {
        Self::new()
    }
}

fn draw_edge((x0, y0): (u16, u16), (x1, y1): (u16, u16), xs: &mut [u16; 200], xi: &mut usize) {
    let dx = x1.abs_diff(x0);
    let dy = y1.abs_diff(y0);

    if dx == 0 && dy == 0 {
        return;
    }

    if dy == 0 {
        xs[*xi] = u16::min(x0, x1);
        *xi += 1;
        return;
    }

    if dx == 0 {
        for _ in y0..=y1 {
            xs[*xi] = x0;
            *xi += 1;
        }
        return;
    }

    let sign_x: i16 = if x0 < x1 { 1 } else { -1 };
    let sign_y: i16 = if y0 < y1 { 1 } else { -1 };

    let bp_6 = sign_y;
    let bp_4 = sign_x;
    let mut bp_2 = sign_y;
    let mut bp_0 = sign_x;

    let mut minor_delta = dy;
    let mut major_delta = dx;

    if dx > dy {
        bp_2 = 0;
    } else {
        swap(&mut minor_delta, &mut major_delta);
        bp_0 = 0;
    }

    let mut x0 = x0;
    let mut ax = major_delta / 2;
    let mut cx = major_delta;
    loop {
        ax += minor_delta;

        let mut dx;
        let bx;
        if ax >= major_delta {
            ax -= major_delta;
            dx = bp_4;
            bx = bp_6;
        } else {
            dx = bp_0;
            bx = bp_2;
        }

        dx += x0 as i16;

        if bx == 1 {
            xs[*xi] = x0;
            *xi += 1;
        }

        x0 = dx as u16;
        cx -= 1;
        if cx == 0 {
            break;
        }
    }
}

fn bresenham_line<F>(p0: (u16, u16), p1: (u16, u16), mut f: F)
where
    F: FnMut(usize, usize),
{
    let mut x0 = p0.0 as i16;
    let mut y0 = p0.1 as i16;
    let mut x1 = p1.0 as i16;
    let mut y1 = p1.1 as i16;

    if x0 > x1 {
        swap(&mut x0, &mut x1);
        swap(&mut y0, &mut y1);
    }

    let dx = i16::abs(x1 - x0);
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -i16::abs(y1 - y0);
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        f(x0 as usize, y0 as usize);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * error;
        if e2 >= dy {
            error += dy;
            x0 += sx;
        }
        if e2 <= dx {
            error += dx;
            y0 += sy;
        }
    }
}
