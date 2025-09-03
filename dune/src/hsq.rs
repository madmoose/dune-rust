use std::io::{Read, Seek, SeekFrom, Write};

use bytes_ext::ReadBytesExt;

#[derive(Debug)]
pub struct Header {
    pub header: [u8; 6],
}

impl Header {
    pub fn from_reader<R: Read>(r: &mut R) -> std::io::Result<Self> {
        let mut header = [0u8; 6];
        r.read_exact(&mut header)?;
        Ok(Header { header })
    }

    pub fn checksum(&self) -> u8 {
        let mut sum = 0u8;
        for i in 0..6 {
            sum = sum.wrapping_add(self.header[i]);
        }
        sum
    }

    pub fn is_compressed(&self) -> bool {
        self.checksum() == 0xAB
    }

    pub fn compressed_size(&self) -> u16 {
        u16::from_le_bytes([self.header[3], self.header[4]])
    }

    pub fn uncompressed_size(&self) -> u32 {
        u32::from_le_bytes([self.header[0], self.header[1], self.header[2], 0])
    }
}

struct Reader<R: Read> {
    queue: u16,
    r: R,
}

impl<R: Read> Reader<R> {
    pub fn read_bit(&mut self) -> bool {
        let mut queue = self.queue;
        let mut bit = (queue & 1) == 1;
        queue >>= 1;
        if queue == 0 {
            queue = self.read_le_u16();
            bit = (queue & 1) == 1;
            queue = 0x8000 | (queue >> 1);
        }
        self.queue = queue;
        bit
    }
    pub fn read_u8(&mut self) -> u8 {
        self.r.read_u8().unwrap()
    }
    pub fn read_le_u16(&mut self) -> u16 {
        self.r.read_le_u16().unwrap()
    }
}

pub fn unhsq<R: Read, W: Read + Write + Seek>(r: R, w: &mut W) -> std::io::Result<()> {
    let mut r = Reader { queue: 0, r };
    let mut w_ofs: u16 = 0;

    loop {
        if r.read_bit() {
            w.seek(SeekFrom::Start(w_ofs as u64))?;
            w.write_all(&[r.read_u8()])?;
            w_ofs += 1;
        } else {
            let mut count: u16;
            let offset: u16;
            if r.read_bit() {
                let word = r.read_le_u16();
                count = word & 7;
                offset = 8192 - (word >> 3);
                if count == 0 {
                    count = r.read_u8() as u16;
                }
                if count == 0 {
                    break;
                }
            } else {
                let b0 = r.read_bit() as u16;
                let b1 = r.read_bit() as u16;

                count = 2 * b0 + b1;
                offset = 256 - (r.read_u8() as u16);
            }

            for _ in 0..count + 2 {
                w.seek(SeekFrom::Start((w_ofs - offset) as u64))?;
                let b = w.read_u8()?;

                w.seek(SeekFrom::Start(w_ofs as u64))?;
                w.write_all(&[b])?;
                w_ofs += 1;
            }
        }
    }

    Ok(())
}
