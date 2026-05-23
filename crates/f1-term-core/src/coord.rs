#[derive(Debug, Default, Clone)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
}

impl Coord {
    pub fn rotate(&self, cos_a: f64, sin_a: f64) -> Coord {
        Coord {
            x: self.x * cos_a - self.y * sin_a,
            y: self.x * sin_a + self.y * cos_a,
        }
    }
}
