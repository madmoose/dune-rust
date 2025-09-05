use std::io::Read;

#[derive(Debug)]
pub struct FrameHeader {
    w: u16,
    h: u8,
    flags: u8,
    mode: u8,
}

impl FrameHeader {
    pub fn new<R: Read>(src: &mut R) -> std::io::Result<Self> {
        /*
         * | w7 w6 w5 w4 w3 w2 w1 w0 | f6 f5 f4 f3 f2 f1 f0 w8 | h7 h6 h5 h4 h3 h2 h1 h0 | m7 m6 m5 m4 m3 m2 m1 m0 |
         */

        let mut b = [0u8; 4];
        src.read_exact(&mut b)?;

        Ok(Self {
            w: ((0x1 & (b[1] as u16)) << 8) | (b[0] as u16),
            h: b[2],
            flags: b[1] & 0xfe,
            mode: b[3],
        })
    }

    pub fn width(&self) -> u16 {
        self.w
    }

    pub fn height(&self) -> u16 {
        self.h as u16
    }

    pub fn is_compressed(&self) -> bool {
        self.flags & 2 != 0
    }

    pub fn is_full_frame(&self) -> bool {
        self.flags & 4 != 0
    }

    pub fn mode(&self) -> u8 {
        self.mode
    }
}
