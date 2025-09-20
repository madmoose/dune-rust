use core::slice;
use std::io::{Read, Write};

use bytes_ext::{ReadBytesExt, WriteBytesExt};

pub fn decompress_rle<R: Read, W: Write>(
    r: &mut R,
    w: &mut W,
    rle_byte: u8,
) -> std::io::Result<()> {
    while let Ok(b) = r.read_u8() {
        if b == rle_byte {
            let cnt = r.read_u8()?;
            let val = r.read_u8()?;
            for _ in 0..cnt {
                w.write_all(slice::from_ref(&val))?;
            }
        } else {
            w.write_all(slice::from_ref(&b))?;
        }
    }

    Ok(())
}

pub fn compress_rle<R: Read, W: Write>(r: &mut R, w: &mut W, rle_byte: u8) -> std::io::Result<()> {
    let mut next_byte = r.read_u8().ok();

    while let Some(current_byte) = next_byte {
        let mut run_count = 1;

        loop {
            next_byte = r.read_u8().ok();

            if next_byte.is_none_or(|b| b != current_byte) || run_count == u8::MAX {
                break;
            }
            run_count += 1;
        }

        if run_count > 2 || current_byte == rle_byte {
            w.write_u8(rle_byte)?;
            w.write_u8(run_count)?;
            w.write_u8(current_byte)?;
            continue;
        }

        w.write_u8(current_byte)?;
        if run_count == 2 {
            w.write_u8(current_byte)?;
        }
    }

    Ok(())
}

#[test]
fn test_compress_no_runs() {
    let data = vec![0x01, 0x02, 0x03];
    let mut input = std::io::Cursor::new(&data);
    let mut output = Vec::new();

    compress_rle(&mut input, &mut output, 0xf7).unwrap();

    assert_eq!(&output, &data);
}

#[test]
fn test_compress_with_rle_byte() {
    let data = vec![0x01, 0xf7, 0x03]; // contains rle byte
    let mut input = std::io::Cursor::new(data);
    let mut output = Vec::new();

    compress_rle(&mut input, &mut output, 0xf7).unwrap();

    // Should be: 0x01 + RLE(count=1, value=0xf7) + 0x03
    assert_eq!(&output, &[0x01, 0xf7, 0x01, 0xf7, 0x03]);
}

#[test]
fn test_compress_with_runs() {
    let data = vec![0x01, 0x01, 0x42, 0x42, 0x42, 0x02];
    let mut input = std::io::Cursor::new(data);
    let mut output = Vec::new();

    compress_rle(&mut input, &mut output, 0xf7).unwrap();

    // Should be: 0x01 + 0x01 + RLE(count=3, value=0x42) + 0x02
    assert_eq!(&output, &[0x01, 0x01, 0xf7, 0x03, 0x42, 0x02]);
}
