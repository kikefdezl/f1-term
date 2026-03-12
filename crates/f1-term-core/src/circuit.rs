use std::{fmt::Display, future::Future, ops::Range};

#[derive(Copy, Debug, Default, Clone, PartialEq)]
pub struct CircuitKey(pub u32);

impl Display for CircuitKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Circuit {
    pub key: CircuitKey,
    pub year: u32,
    pub short_name: String,
    pub layout: Option<CircuitLayout>,
    pub status: CircuitStatus,
}

#[derive(Debug, Default, Clone)]
pub enum CircuitStatus {
    // Clear and Red are always on the full circuit and don't need a scope
    #[default]
    Clear,
    Red,
    Yellow(CircuitScope),
}

#[derive(Debug, Default, Clone)]
pub enum CircuitScope {
    #[default]
    Full,
    Sectors(Vec<u8>),
}

#[derive(Debug, Default, Clone)]
pub struct CircuitLayout {
    pub coords: Vec<Coord>,
    pub rotation: f64,
    pub corners: Vec<Corner>,
    // Each mini sector is a range of indexes of the coords that it contains
    pub mini_sectors: Vec<Range<usize>>,
}

impl CircuitLayout {
    pub fn rotate(&self) -> CircuitLayout {
        let angle_rad = self.rotation.to_radians();
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        let mut coords_rot = Vec::with_capacity(self.coords.len());
        for i in 0..self.coords.len() {
            coords_rot.push(self.coords[i].rotate(cos_a, sin_a));
        }

        let mut corners_rot = Vec::with_capacity(self.corners.len());
        for corner in &self.corners {
            corners_rot.push(Corner {
                num: corner.num,
                coord: corner.coord.rotate(cos_a, sin_a),
            })
        }

        CircuitLayout {
            coords: coords_rot,
            rotation: 0.0,
            corners: corners_rot,
            mini_sectors: self.mini_sectors.clone(),
        }
    }

    pub fn bounds(&self) -> Bounds {
        let (mut x_min, mut y_min) = (f64::MAX, f64::MAX);
        let (mut x_max, mut y_max) = (f64::MIN, f64::MIN);
        for c in &self.coords {
            if c.x > x_max {
                x_max = c.x;
            }
            if c.x < x_min {
                x_min = c.x;
            }
            if c.y > y_max {
                y_max = c.y;
            }
            if c.y < y_min {
                y_min = c.y;
            }
        }
        Bounds {
            x_min: x_min.floor() as i32,
            y_min: y_min.floor() as i32,
            x_max: x_max.ceil() as i32,
            y_max: y_max.ceil() as i32,
        }
    }
}

#[derive(Default)]
pub struct Bounds {
    pub x_min: i32,
    pub y_min: i32,
    pub x_max: i32,
    pub y_max: i32,
}

pub trait CircuitLayoutProvider: Send + Sync {
    fn fetch(
        &self,
        circuit_key: CircuitKey,
        year: u32,
    ) -> impl Future<Output = Result<CircuitLayout, Box<dyn std::error::Error>>> + Send;
}

#[derive(Clone, Debug)]
pub struct Corner {
    pub num: u8,
    pub coord: Coord,
}

#[derive(Debug, Default, Clone)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
}

impl Coord {
    fn rotate(&self, cos_a: f64, sin_a: f64) -> Coord {
        Coord {
            x: self.x * cos_a - self.y * sin_a,
            y: self.x * sin_a + self.y * cos_a,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate() {
        let layout = CircuitLayout {
            coords: vec![
                Coord { x: 100.0, y: 0.0 },
                Coord { x: 0.0, y: 100.0 },
                Coord { x: -100.0, y: 0.0 },
                Coord { x: 0.0, y: -100.0 },
            ],
            rotation: 90.0,
            corners: Vec::new(),
            mini_sectors: vec![Range { start: 0, end: 3 }],
        };

        let rotated = layout.rotate();

        assert_eq!(rotated.coords.len(), 4);

        // 90 degrees rotation (counter-clockwise)
        // (100, 0) -> (0, 100)
        assert!((rotated.coords[0].x - 0.0).abs() < 1e-6);
        assert!((rotated.coords[0].y - 100.0).abs() < 1e-6);

        // (0, 100) -> (-100, 0)
        assert!((rotated.coords[1].x - (-100.0)).abs() < 1e-6);
        assert!((rotated.coords[1].y - 0.0).abs() < 1e-6);

        // (-100, 0) -> (0, -100)
        assert!((rotated.coords[2].x - 0.0).abs() < 1e-6);
        assert!((rotated.coords[2].y - (-100.0)).abs() < 1e-6);

        // (0, -100) -> (100, 0)
        assert!((rotated.coords[3].x - 100.0).abs() < 1e-6);
        assert!((rotated.coords[3].y - 0.0).abs() < 1e-6);
    }
}
