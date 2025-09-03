#[derive(Copy, Clone, Debug, Default)]
pub struct Rect {
    pub x0: i16,
    pub y0: i16,
    pub x1: i16,
    pub y1: i16,
}

impl Rect {
    pub fn default_clip_rect() -> Self {
        Rect {
            x0: 0,
            y0: 0,
            x1: 320,
            y1: 200,
        }
    }

    pub fn clip(&self, bounds: &Rect) -> Rect {
        let x0 = self.x0.clamp(bounds.x0, bounds.x1);
        let y0 = self.y0.clamp(bounds.y0, bounds.y1);
        let x1 = self.x1.clamp(bounds.x0, bounds.x1);
        let y1 = self.y1.clamp(bounds.y0, bounds.y1);

        Rect { x0, y0, x1, y1 }
    }

    pub fn in_rect(&self, x: i16, y: i16) -> bool {
        x >= self.x0 && x < self.x1 && y >= self.y0 && y < self.y1
    }

    pub fn is_empty(&self) -> bool {
        self.x1 <= self.x0 || self.y1 <= self.y0
    }
}
