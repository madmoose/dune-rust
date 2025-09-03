#[derive(Debug)]
pub struct IndexMap(pub [u8; 320 * 200]);

impl IndexMap {
    pub fn new() -> IndexMap {
        IndexMap([255; 320 * 200])
    }

    pub fn clear(&mut self) {
        self.0.fill(0xff);
    }

    pub fn set_index(&mut self, x: usize, y: usize, index: usize) {
        if x < 320 && y < 200 {
            self.0[y * 320 + x] = index as u8;
        }
    }

    pub fn get_index(&self, x: usize, y: usize) -> Option<usize> {
        if x < 320 && y < 200 {
            let v = self.0[y * 320 + x];
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
