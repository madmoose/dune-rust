pub trait WriteBytesExt: std::io::Write {
    #[inline]
    fn write_u8(&mut self, v: u8) -> std::io::Result<()> {
        let buf = v.to_le_bytes();
        self.write_all(&buf)?;
        Ok(())
    }

    #[inline]
    fn write_le_u16(&mut self, v: u16) -> std::io::Result<()> {
        let buf = v.to_le_bytes();
        self.write_all(&buf)?;
        Ok(())
    }
}

impl<W: std::io::Write> WriteBytesExt for W {}
