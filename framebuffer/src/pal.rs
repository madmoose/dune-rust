#[derive(Debug)]
pub struct Pal([(u8, u8, u8); 256]);

impl Pal {
    pub fn new() -> Self {
        let mut pal = [(0u8, 0u8, 0u8); 256];

        for i in 0..256 {
            pal[i + 0] = (0, 0, 0);
        }

        Pal(pal)
    }

    pub fn clear(&mut self) {
        for i in 0..256 {
            self.set(i, (0, 0, 0));
        }
    }

    pub fn get(&self, i: usize) -> (u8, u8, u8) {
        self.0[i]
    }

    pub fn set(&mut self, i: usize, rgb: (u8, u8, u8)) {
        self.0[i] = rgb;
    }

    pub fn set_all(&mut self, pal: &[u8; 768]) {
        for i in 0..256 {
            self.set(i, (pal[3 * i + 0], pal[3 * i + 1], pal[3 * i + 2]))
        }
    }

    pub fn as_slice(&self) -> &[(u8, u8, u8); 256] {
        &self.0
    }

    pub fn as_mut_slice(&mut self) -> &mut [(u8, u8, u8); 256] {
        &mut self.0
    }
}

impl Default for Pal {
    fn default() -> Self {
        Self::new()
    }
}
