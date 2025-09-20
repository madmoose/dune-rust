use std::io::{Cursor, Error, ErrorKind};

use bytes_ext::{ReadBytesExt, WriteBytesExt};

use crate::rle::{compress_rle, decompress_rle};

pub mod data;
mod rle;

const RLE_BYTE: u8 = 0xf7;

pub struct UnparsedSavegame {
    pub gametime: u16,
    pub data: Vec<u8>,
}

pub fn decompress_sav(input: &[u8]) -> std::io::Result<UnparsedSavegame> {
    if input.len() < 6 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "invalid save game: too short",
        ));
    }

    let mut r = Cursor::new(input);

    let gametime = r.read_le_u16()?;
    let rle = r.read_le_u16()?;
    let len = r.read_le_u16()?;

    if len as usize != input.len() - 2 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "invalid header: header indicates {} bytes, got {} bytes",
                len,
                input.len()
            ),
        ));
    }

    let mut w = Vec::new();
    decompress_rle(&mut r, &mut w, rle as u8)?;

    Ok(UnparsedSavegame { gametime, data: w })
}

pub fn compress_sav(input: &[u8], gametime: u16) -> std::io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut w = Cursor::new(&mut output);

    w.write_le_u16(gametime)?;
    w.write_le_u16(RLE_BYTE as u16)?;
    w.write_le_u16(0)?;

    let mut r = Cursor::new(input);
    compress_rle(&mut r, &mut w, RLE_BYTE)?;

    let len = w.position() - 2;
    w.set_position(4);
    w.write_le_u16(len as u16)?;

    Ok(output)
}
