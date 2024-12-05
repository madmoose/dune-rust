use std::io::Cursor;

use bytes_ext::ReadBytesExt;

use crate::{
    room::{Character, Line, Part, Polygon, Sprite},
    GaloisNoiseGenerator, Room,
};

#[derive(Debug)]
pub struct RoomSheet {
    rooms: Vec<Room>,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    FormatError(&'static str),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

impl RoomSheet {
    pub fn new(data: &[u8]) -> Result<Self, Error> {
        let mut r = Cursor::new(data);

        let room_0_ofs = r.read_le_u16()?;
        let room_count = room_0_ofs / 2;
        if room_count == 0 {
            return Result::Err(Error::FormatError("invalid room count"));
        }

        let mut room_offsets = Vec::with_capacity(room_count.into());
        room_offsets.push(room_0_ofs);

        for _ in 1..room_count {
            room_offsets.push(r.read_le_u16()?);
        }

        let mut rooms = Vec::with_capacity(room_count.into());
        for ofs in room_offsets {
            r.set_position(ofs.into());

            let _position_marker_count = r.read_u8()?;
            let mut room = Room::new();

            loop {
                let cmd = r.read_le_u16()?;
                if cmd == 0xffff {
                    break;
                }

                if (cmd & 0x8000) == 0 {
                    let x = (r.read_u8()? as u16) + if (cmd & 0x0200) != 0 { 256 } else { 0 };
                    let y = r.read_u8()?;
                    let pal_offset = r.read_u8()?;

                    if (cmd & 0x1ff) == 1 {
                        room.add_part(Part::Character(Character { x, y, pal_offset }));
                    } else {
                        room.add_part(Part::Sprite(Sprite {
                            id: (cmd & 0x1ff) - 1,
                            x,
                            y,
                            flip_x: cmd & 0x4000 != 0,
                            flip_y: cmd & 0x2000 != 0,
                            scale: ((cmd >> 10) & 7) as u8,
                            pal_offset,
                        }));
                    }
                } else if (cmd & 0x4000) == 0 {
                    // Polygon

                    let noise_seed = ((cmd & 0x3e00) != 0) as u16;
                    let noise_mask = (cmd & 0x3e00) | 2;

                    let reverse_gradient = (cmd & 0x100) != 0;
                    let h_gradient = 16 * (r.read_i8()? as i16);
                    let v_gradient = 16 * (r.read_i8()? as i16);

                    let start_x = r.read_le_u16()?;
                    let start_y = r.read_le_u16()?;

                    let mut x;
                    let mut y;

                    let mut right_vertices = Vec::new();
                    let mut left_vertices = Vec::new();

                    right_vertices.push((start_x, start_y));

                    loop {
                        x = r.read_le_u16()?;
                        y = r.read_le_u16()?;

                        right_vertices.push((x & 0x3fff, y));

                        if x & 0x4000 != 0 {
                            break;
                        }
                    }

                    if x & 0x8000 == 0 {
                        loop {
                            x = r.read_le_u16()?;
                            y = r.read_le_u16()?;

                            left_vertices.push((x & 0x3fff, y));

                            if x & 0x8000 != 0 {
                                break;
                            }
                        }
                    }

                    room.add_part(Part::Polygon(Polygon {
                        right_vertices,
                        left_vertices,
                        h_gradient,
                        v_gradient,
                        reverse_gradient,
                        color: (cmd & 0xff) as u8,
                        noise: GaloisNoiseGenerator {
                            state: noise_seed,
                            mask: noise_mask,
                        },
                    }));
                } else {
                    // Line
                    let p0 = (r.read_le_u16()?, r.read_le_u16()?);
                    let p1 = (r.read_le_u16()?, r.read_le_u16()?);
                    room.add_part(Part::Line(Line {
                        p0,
                        p1,
                        color: (cmd & 0xff) as u8,
                        dither: 0xffffu16,
                    }));
                }
            }

            rooms.push(room);
        }

        Ok(RoomSheet { rooms })
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    pub fn get_room(&self, room: usize) -> Option<&Room> {
        self.rooms.get(room)
    }
}
