#![allow(clippy::too_many_arguments)]

#[derive(Clone)]
pub struct Sprite {
    width: u16,
    height: u16,
    pal_offset: u8,
    rle: bool,
    data: Vec<u8>,
}

impl std::fmt::Debug for Sprite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sprite")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pal_offset", &self.pal_offset)
            .field("rle", &self.rle)
            // .field_with("data", |fmt| {
            //     fmt.write_str("[")?;
            //     for b in self.data.bytes().take(16) {
            //         write!(fmt, " {:#04x}", b.unwrap())?;
            //     }
            //     if self.data.len() > 16 {
            //         write!(fmt, " ...")?;
            //     }
            //     fmt.write_str(" ]")?;
            //     Ok(())
            // })
            .finish()
    }
}

impl Sprite {
    pub fn from_slice(data: &[u8]) -> Option<Self> {
        let w0 = u16::from_le_bytes(data[0..2].try_into().ok()?);
        let w1 = u16::from_le_bytes(data[2..4].try_into().ok()?);
        let data = Vec::from(&data[4..]);

        let flags = (w0 & 0xfe00) >> 8;
        let width = w0 & 0x01ff;
        let pal_offset = ((w1 & 0xff00) >> 8) as u8;
        let height = w1 & 0x00ff;

        if width == 0 || height == 0 || width > (i16::MAX as u16) || height > (i16::MAX as u16) {
            return None;
        }

        let rle = (flags & 0x80) != 0;

        Some(Sprite {
            width,
            height,
            pal_offset,
            rle,
            data,
        })
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn pal_offset(&self) -> u8 {
        self.pal_offset
    }

    pub fn set_pal_offset(&mut self, pal_offset: u8) {
        self.pal_offset = pal_offset;
    }

    pub fn rle(&self) -> bool {
        self.rle
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
