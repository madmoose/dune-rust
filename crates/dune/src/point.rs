#[derive(Clone)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl From<(i16, i16)> for Point {
    fn from((x, y): (i16, i16)) -> Self {
        Point { x, y }
    }
}
