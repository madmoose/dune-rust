#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn as_i16(self) -> (i16, i16, i16) {
        (self.0 as i16, self.1 as i16, self.2 as i16)
    }

    pub fn from_i16(r: i16, g: i16, b: i16) -> Self {
        Color(r as u8, g as u8, b as u8)
    }

    pub fn lerp(self, other: Color, divisor: i16) -> Self {
        let (r0, g0, b0) = self.as_i16();
        let (r1, g1, b1) = other.as_i16();

        let r = (r1 - r0) / divisor + r0;
        let g = (g1 - g0) / divisor + g0;
        let b = (b1 - b0) / divisor + b0;

        Color::from_i16(r, g, b)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(t: (u8, u8, u8)) -> Self {
        Color(t.0, t.1, t.2)
    }
}

impl From<Color> for (u8, u8, u8) {
    fn from(c: Color) -> Self {
        (c.0, c.1, c.2)
    }
}
