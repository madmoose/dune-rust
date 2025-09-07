#[derive(Clone)]
pub struct Image<T> {
    pub(crate) w: u16,
    pub(crate) h: u16,
    pub(crate) pixels: Box<[T]>,
}

impl<T> Image<T>
where
    T: Copy + Default,
{
    pub fn new(w: u16, h: u16) -> Self {
        let pixels = vec![T::default(); w as usize * h as usize].into_boxed_slice();
        Self { w, h, pixels }
    }

    pub fn w(&self) -> u16 {
        self.w
    }

    pub fn h(&self) -> u16 {
        self.h
    }

    pub fn pixels(&self) -> &[T] {
        &self.pixels
    }

    pub fn clear(&mut self) {
        self.pixels.fill(T::default());
    }

    pub fn set(&mut self, x: u16, y: u16, c: T) {
        self.pixels[y as usize * self.w as usize + x as usize] = c;
    }

    pub fn get(&self, x: u16, y: u16) -> T {
        self.pixels[y as usize * self.w as usize + x as usize]
    }

    pub fn copy_from(&mut self, other: &Self) {
        assert_eq!(self.w, other.w);
        assert_eq!(self.h, other.h);
        self.pixels.copy_from_slice(&other.pixels);
    }
}
