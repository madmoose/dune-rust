use crate::Framebuffer;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextSize {
    #[default]
    Small,
    Large,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

pub struct Font {
    data: Box<[u8]>,
}

impl Font {
    pub fn new(data: &[u8]) -> Self {
        Self { data: data.into() }
    }
}

pub struct TextContext<'a> {
    font: &'a Font,
    framebuffer: &'a mut Framebuffer,
}

impl<'a> TextContext<'a> {
    pub fn new(font: &'a Font, framebuffer: &'a mut Framebuffer) -> Self {
        Self { font, framebuffer }
    }

    pub fn draw_text(&mut self, style: TextStyle, x: u16, y: u16, s: &str) {
        draw_text(self.font, self.framebuffer, style, x, y, s);
    }

    pub fn measure_text(&self, style: TextStyle, s: &str) -> u16 {
        style.measure_text(self.font, s)
    }
}

#[derive(Copy, Clone, Default)]
pub struct TextStyle {
    pub size: TextSize,
    pub color: u8,
    pub align: TextAlign,
}

impl TextStyle {
    pub fn new() -> Self {
        Self {
            size: TextSize::Small,
            color: 0,
            align: TextAlign::Left,
        }
    }

    pub fn size(mut self) -> Self {
        self.size = TextSize::Large;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = TextSize::Small;
        self
    }

    pub fn large(mut self) -> Self {
        self.size = TextSize::Large;
        self
    }

    pub fn color(mut self, color: u8) -> Self {
        self.color = color;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn left(mut self) -> Self {
        self.align = TextAlign::Left;
        self
    }

    pub fn center(mut self) -> Self {
        self.align = TextAlign::Center;
        self
    }

    pub fn right(mut self) -> Self {
        self.align = TextAlign::Right;
        self
    }

    pub fn measure_text(&self, font: &Font, s: &str) -> u16 {
        s.chars()
            .map(|c| glyph_width(font, c, self.size) as u16)
            .sum()
    }
}

pub fn draw_text(
    font: &Font,
    framebuffer: &mut Framebuffer,
    style: TextStyle,
    x: u16,
    y: u16,
    s: &str,
) {
    let w = style.measure_text(font, s);

    let mut x = match style.align {
        TextAlign::Left => x,
        TextAlign::Center => x - w / 2,
        TextAlign::Right => x - w,
    };

    for c in s.chars() {
        draw_glyph(font, framebuffer, x, y, c, style.size, style.color);
        x += glyph_width(font, c, style.size) as u16;
    }
}

fn draw_glyph(
    font: &Font,
    framebuffer: &mut Framebuffer,
    x: u16,
    y: u16,
    c: char,
    size: TextSize,
    color: u8,
) {
    let mut glyph_ofs = match size {
        TextSize::Large => 0x100 + glyph_height(size) as usize * (c as usize),
        TextSize::Small => 0x580 + glyph_height(size) as usize * (c as usize),
    };

    let h = glyph_height(size) as u16;
    let w = glyph_width(font, c, size) as u16;

    for y in y..y + h {
        let mut mask = 0x80;
        for x in x..x + w {
            if font.data[glyph_ofs] & mask != 0 {
                framebuffer.set(x, y, color);
            }
            mask >>= 1;
        }
        glyph_ofs += 1;
    }
}

fn glyph_width(font: &Font, c: char, size: TextSize) -> u8 {
    match size {
        TextSize::Large => font.data[c as usize],
        TextSize::Small => font.data[c as usize + 0x80],
    }
}

fn glyph_height(size: TextSize) -> u8 {
    match size {
        TextSize::Large => 9,
        TextSize::Small => 7,
    }
}
