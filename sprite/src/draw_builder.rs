use dune::{
    blit::{self},
    Rect,
};
use framebuffer::{Framebuffer, IndexMap};

use crate::Sprite;

pub struct DrawBuilder<'a> {
    sprite: &'a Sprite,
    framebuffer: &'a mut Framebuffer,
    index: usize,
    x: i16,
    y: i16,
    clip_rect: Rect,
    flip_x: bool,
    flip_y: bool,
    scale: u8,
    pal_offset: u8,
    index_map: Option<IndexMap>,
}

impl<'a> DrawBuilder<'a> {
    pub fn new(sprite: &'a Sprite, framebuffer: &'a mut Framebuffer) -> Self {
        Self {
            sprite,
            framebuffer,
            index: 0,
            x: 0,
            y: 0,
            clip_rect: Rect {
                x0: 0,
                y0: 0,
                x1: 320,
                y1: 200,
            },
            flip_x: false,
            flip_y: false,
            scale: 0,
            pal_offset: 0,
            index_map: None,
        }
    }

    pub fn at(mut self, x: i16, y: i16) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn clip_rect(mut self, clip_rect: Rect) -> Self {
        self.clip_rect = clip_rect;
        self
    }

    pub fn flip_x(mut self, flip_x: bool) -> Self {
        self.flip_x = flip_x;
        self
    }

    pub fn flip_y(mut self, flip_y: bool) -> Self {
        self.flip_y = flip_y;
        self
    }

    pub fn scale(mut self, scale: u8) -> Self {
        self.scale = scale;
        self
    }

    pub fn pal_offset(mut self, pal_offset: u8) -> Self {
        self.pal_offset = pal_offset;
        self
    }

    pub fn index_map(mut self, index_map: IndexMap) -> Self {
        self.index_map = Some(index_map);
        self
    }

    pub fn opt_index_map(mut self, index_map: Option<IndexMap>) -> Self {
        self.index_map = index_map;
        self
    }

    pub fn draw(self) -> std::io::Result<()> {
        let mut index_map_option = self.index_map.map(Some).unwrap_or(None);

        let mut pal_offset = self.pal_offset;
        if pal_offset == 0 {
            pal_offset = self.sprite.pal_offset();
        }

        blit::draw_with_options(
            self.index,
            self.x,
            self.y,
            self.sprite.width(),
            self.sprite.height(),
            self.clip_rect,
            false,
            self.flip_x,
            self.flip_y,
            self.scale,
            pal_offset,
            self.sprite.data(),
            self.framebuffer,
            &mut index_map_option,
        )
    }
}
