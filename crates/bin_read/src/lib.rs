extern crate bin_read_derive;

pub use bin_read_derive::BinRead;

pub trait BinRead: Sized {
    fn bin_read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
    ) -> Result<Self, Box<dyn std::error::Error>>;
}

// Implementation for u8
impl BinRead for u8 {
    fn bin_read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

// Implementation for u16 (little-endian)
impl BinRead for u16 {
    fn bin_read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }
}

// Implementation for i16 (little-endian)
impl BinRead for i16 {
    fn bin_read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }
}

// Implementation for u32 (little-endian)
impl BinRead for u32 {
    fn bin_read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}

// Implementation for u64 (little-endian)
impl BinRead for u64 {
    fn bin_read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
}

// Implementation for arrays of fixed size
impl<T: BinRead, const N: usize> BinRead for [T; N] {
    fn bin_read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut array: [std::mem::MaybeUninit<T>; N] =
            unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        for e in &mut array {
            *e = std::mem::MaybeUninit::new(T::bin_read(reader)?);
        }

        // Safety: We've initialized all elements
        Ok(unsafe { std::mem::transmute_copy::<[std::mem::MaybeUninit<T>; N], [T; N]>(&array) })
    }
}

// Implementation for Vec<T> - requires a length prefix (u32)
impl<T: BinRead> BinRead for Vec<T> {
    fn bin_read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let len = u32::bin_read(reader)? as usize;
        let mut vec = Vec::with_capacity(len);

        for _ in 0..len {
            vec.push(T::bin_read(reader)?);
        }

        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use bin_read_derive::BinRead;

    use super::*;

    #[derive(BinRead, Debug, PartialEq)]
    struct SimpleStruct {
        field1: u8,
        field2: u16,
    }

    #[derive(BinRead, Debug, PartialEq)]
    struct NestedStruct {
        simple: SimpleStruct,
        array: [u8; 3],
    }

    #[derive(BinRead, Debug, PartialEq)]
    struct TupleStruct(u8, u16);

    #[test]
    fn test_simple_struct() {
        let data = vec![0x42, 0x34, 0x12]; // u8: 0x42, u16: 0x1234 (little-endian)
        let mut cursor = Cursor::new(data);

        let result = SimpleStruct::bin_read(&mut cursor).unwrap();
        assert_eq!(
            result,
            SimpleStruct {
                field1: 0x42,
                field2: 0x1234
            }
        );
    }

    #[test]
    fn test_nested_struct() {
        let data = vec![
            0x42, 0x34, 0x12, // SimpleStruct
            0x01, 0x02, 0x03, // [u8; 3]
        ];
        let mut cursor = Cursor::new(data);

        let result = NestedStruct::bin_read(&mut cursor).unwrap();
        assert_eq!(
            result,
            NestedStruct {
                simple: SimpleStruct {
                    field1: 0x42,
                    field2: 0x1234
                },
                array: [0x01, 0x02, 0x03],
            }
        );
    }

    #[test]
    fn test_tuple_struct() {
        let data = vec![0x42, 0x34, 0x12]; // u8: 0x42, u16: 0x1234 (little-endian)
        let mut cursor = Cursor::new(data);

        let result = TupleStruct::bin_read(&mut cursor).unwrap();
        assert_eq!(result, TupleStruct(0x42, 0x1234));
    }

    #[test]
    fn test_array() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut cursor = Cursor::new(data);

        let result = <[u8; 4]>::bin_read(&mut cursor).unwrap();
        assert_eq!(result, [0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_vec() {
        let data = vec![
            0x03, 0x00, 0x00, 0x00, // length: 3 (u32 little-endian)
            0x01, 0x02, 0x03, // data
        ];
        let mut cursor = Cursor::new(data);

        let result = Vec::<u8>::bin_read(&mut cursor).unwrap();
        assert_eq!(result, vec![0x01, 0x02, 0x03]);
    }

    #[derive(BinRead, Debug, PartialEq)]
    struct StructWithOffset {
        field1: u8,
        #[bin_read(offset = 5)]
        field2: u16,
        field3: u8,
    }

    #[test]
    fn test_offset_attribute() {
        let data = vec![
            0x42, // offset 0: field1 (u8)
            0xFF, 0xFF, 0xFF, // offset 1-3: padding/junk
            0xFF, // offset 4: padding/junk
            0x34, 0x12, // offset 5-6: field2 (u16, little-endian)
            0x99, // offset 7: field3 (u8)
        ];
        let mut cursor = Cursor::new(data);

        let result = StructWithOffset::bin_read(&mut cursor).unwrap();
        assert_eq!(
            result,
            StructWithOffset {
                field1: 0x42,
                field2: 0x1234,
                field3: 0x99,
            }
        );
    }
}
