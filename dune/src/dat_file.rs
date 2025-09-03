use std::{
    fs::File,
    io::{BufReader, Cursor, ErrorKind, Read, Seek},
    path::Path,
};

use bytes_ext::ReadBytesExt;

use crate::hsq;

pub struct DatFile {
    reader: BufReader<File>,
    pub entries: Vec<DatEntry>,
}

#[derive(Debug)]
pub struct DatEntry {
    pub name: String,
    pub offset: usize,
    pub size: usize,
}

type Error = std::io::Error;

impl DatFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<DatFile, Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let entry_count = reader.read_le_u16()? as usize;
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            let name = reader.read_fixed_str(16)?;
            let size = reader.read_le_u32()? as usize;
            let offset = reader.read_le_u32()? as usize;
            _ = reader.read_u8();

            if name.is_empty() {
                break;
            }

            entries.push(DatEntry { name, size, offset });
        }

        Ok(DatFile { reader, entries })
    }

    pub fn read_raw(&mut self, name: &str) -> Result<Vec<u8>, Error> {
        let entry = self
            .entries
            .iter()
            .find(|&e| e.name == name)
            .ok_or(Error::from(ErrorKind::NotFound))?;

        self.reader
            .seek(std::io::SeekFrom::Start(entry.offset as u64))?;

        let mut data = vec![0; entry.size];
        self.reader.read_exact(data.as_mut_slice())?;

        Ok(data)
    }

    pub fn read(&mut self, name: &str) -> Result<Vec<u8>, Error> {
        let data = self.read_raw(name)?;

        let mut reader = Cursor::new(&data);
        let header = hsq::Header::from_reader(&mut reader)?;

        if !header.is_compressed() {
            return Ok(data);
        }

        if header.compressed_size() as usize != data.len() {
            println!("Packed length does not match resource size");
            return Ok(data);
        }

        let mut unpacked_data = vec![0; header.uncompressed_size() as usize];
        let mut writer = Cursor::new(&mut unpacked_data);

        hsq::unhsq(&data[6..], &mut writer).unwrap();
        Ok(unpacked_data)
    }
}
