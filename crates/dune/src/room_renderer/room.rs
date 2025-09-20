use serde::Serialize;

use crate::room_renderer::galois_noise_generator::GaloisNoiseGenerator;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub struct Room {
    _position_marker_count: u8,
    parts: Vec<Part>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub struct Sprite {
    pub id: u16,
    pub x: u16,
    pub y: u8,
    pub flip_x: bool,
    pub flip_y: bool,
    pub scale: u8,
    pub pal_offset: u8,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub struct Character {
    pub x: u16,
    pub y: u8,
    pub pal_offset: u8,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub struct Polygon {
    pub right_vertices: Vec<(i16, i16)>,
    pub left_vertices: Vec<(i16, i16)>,
    pub h_gradient: i16,
    pub v_gradient: i16,
    pub reverse_gradient: bool,
    pub color: u8,
    pub noise: GaloisNoiseGenerator,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub struct Line {
    pub p0: (i16, i16),
    pub p1: (i16, i16),
    pub color: u8,
    pub dither: u16,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum Part {
    Sprite(Sprite),
    Character(Character),
    Polygon(Polygon),
    Line(Line),
}

impl Room {
    pub fn new() -> Self {
        Self {
            _position_marker_count: 0,
            parts: Vec::new(),
        }
    }

    pub fn parts(&self) -> &[Part] {
        &self.parts
    }

    pub fn add_part(&mut self, part: Part) {
        self.parts.push(part);
    }

    pub fn remove_part(&mut self, index: usize) {
        self.parts.remove(index);
    }
}

impl Default for Room {
    fn default() -> Self {
        Self::new()
    }
}
