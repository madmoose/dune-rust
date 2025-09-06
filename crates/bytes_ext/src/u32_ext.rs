pub trait U32Ext {
    fn low_word(&self) -> u16;
    fn high_word(&self) -> u16;
    fn from_le_words(words: [u16; 2]) -> u32;
    fn to_le_words(&self) -> [u16; 2];
    fn from_be_words(words: [u16; 2]) -> u32;
    fn to_be_words(&self) -> [u16; 2];
}

impl U32Ext for u32 {
    fn low_word(&self) -> u16 {
        *self as u16
    }

    fn high_word(&self) -> u16 {
        (*self >> 16) as u16
    }

    fn from_le_words(words: [u16; 2]) -> u32 {
        (words[1] as u32) << 16 | (words[0] as u32)
    }

    fn to_le_words(&self) -> [u16; 2] {
        [self.low_word(), self.high_word()]
    }

    fn from_be_words(words: [u16; 2]) -> u32 {
        (words[0] as u32) << 16 | (words[1] as u32)
    }

    fn to_be_words(&self) -> [u16; 2] {
        [self.high_word(), self.low_word()]
    }
}
