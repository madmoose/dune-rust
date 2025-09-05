mod read_bytes_ext;
mod write_bytes_ext;

pub use read_bytes_ext::ReadBytesExt;
pub use write_bytes_ext::WriteBytesExt;

pub trait U16Manip<'a> {
    fn hi(self) -> u8;
    fn hi_mut(&'a mut self) -> U16Hi<'a>;
    fn lo(self) -> u8;
}

pub struct U16Hi<'a>(&'a mut u16);

impl<'a> U16Hi<'a> {
    pub fn get(&self) -> u8 {
        (*self.0 >> 8) as u8
    }

    pub fn set(&mut self, value: u8) {
        let lo = *self.0 & 0x00ff;
        *self.0 = ((value as u16) << 8) | lo;
    }

    pub fn rotate_left(&mut self, n: u32) {
        *self.0 = self.0.rotate_left(n);
    }

    pub fn rotate_right(&mut self, n: u32) {
        *self.0 = self.0.rotate_right(n);
    }
}

impl<'a> U16Manip<'a> for u16 {
    fn hi(self) -> u8 {
        (self >> 8) as u8
    }

    fn hi_mut(&'a mut self) -> U16Hi<'a> {
        U16Hi(self)
    }

    fn lo(self) -> u8 {
        (self & 0x00ff) as u8
    }
}
