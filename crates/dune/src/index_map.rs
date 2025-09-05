#[derive(Debug)]
pub struct IndexMap(pub [u8; 320 * 200]);

impl IndexMap {
    pub fn new() -> IndexMap {
        IndexMap([255; 320 * 200])
    }

    pub fn clear(&mut self) {
        self.0.fill(0xff);
    }

    pub fn set_index(&mut self, x: u16, y: u16, index: usize) {
        if x < 320 && y < 200 {
            self.0[y as usize * 320 + x as usize] = index as u8;
        }
    }

    pub fn get_index(&self, x: u16, y: u16) -> Option<usize> {
        if (0..320).contains(&x) && (0..200).contains(&y) {
            let v = self.0[(y * 320 + x) as usize];
            if v < 255 {
                return Some(v as usize);
            }
        }
        None
    }
}

impl Default for IndexMap {
    fn default() -> Self {
        Self::new()
    }
}
