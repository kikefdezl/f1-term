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

#[derive(Debug, Default, Clone)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
}

impl CircuitLayout {
    pub fn rotated_points(&self) -> (Vec<f64>, Vec<f64>) {
        let angle_rad = self.rotation.to_radians();
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        let mut x_rot = Vec::with_capacity(self.coords.len());
        let mut y_rot = Vec::with_capacity(self.coords.len());

        for i in 0..self.coords.len() {
            let x = self.coords[i].x;
            let y = self.coords[i].y;
            x_rot.push(x * cos_a - y * sin_a);
            y_rot.push(x * sin_a + y * cos_a);
        }
        (x_rot, y_rot)
    }

    pub fn bounds(&self) -> Bounds {
        let (x_rot, y_rot) = self.rotated_points();
        let (mut x_min, mut y_min) = (f64::MAX, f64::MAX);
        let (mut x_max, mut y_max) = (f64::MIN, f64::MIN);
        for &x in &x_rot {
            if x > x_max {
                x_max = x;
            }
            if x < x_min {
                x_min = x;
            }
        }
        for &y in &y_rot {
            if y > y_max {
                y_max = y;
            }
            if y < y_min {
                y_min = y;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotated_points() {
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

        let (x_rot, y_rot) = layout.rotated_points();

        assert_eq!(x_rot.len(), 4);
        assert_eq!(y_rot.len(), 4);

        // 90 degrees rotation (counter-clockwise)
        // (100, 0) -> (0, 100)
        assert!((x_rot[0] - 0.0).abs() < 1e-6);
        assert!((y_rot[0] - 100.0).abs() < 1e-6);

        // (0, 100) -> (-100, 0)
        assert!((x_rot[1] - (-100.0)).abs() < 1e-6);
        assert!((y_rot[1] - 0.0).abs() < 1e-6);

        // (-100, 0) -> (0, -100)
        assert!((x_rot[2] - 0.0).abs() < 1e-6);
        assert!((y_rot[2] - (-100.0)).abs() < 1e-6);

        // (0, -100) -> (100, 0)
        assert!((x_rot[3] - 100.0).abs() < 1e-6);
        assert!((y_rot[3] - 0.0).abs() < 1e-6);
    }
}
