use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct GaloisNoiseGenerator {
    pub state: u16,
    pub mask: u16,
}

impl GaloisNoiseGenerator {
    pub fn rand(&mut self) -> u16 {
        let lsb = (self.state & 1) != 0;
        self.state >>= 1;
        if lsb {
            self.state ^= self.mask;
        }
        self.state
    }
}
