use std::future::Future;

#[derive(Debug, Default, Clone)]
pub struct Circuit {
    pub key: u32,
    pub short_name: String,
    pub layout: Option<CircuitLayout>,
}

#[derive(Debug, Default, Clone)]
pub struct CircuitLayout {
    pub coords: Vec<Coord>,
    pub rotation: f64,
    pub corners: Vec<Corner>,
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
pub trait CircuitLayoutProvider {
    fn fetch(
        &self,
        circuit_key: u32,
        year: u32,
    ) -> impl Future<Output = anyhow::Result<CircuitLayout>> + Send;
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
