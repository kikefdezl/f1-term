use std::future::Future;

#[derive(Debug, Default, Clone)]
pub struct Circuit {
    pub key: u32,
    pub short_name: String,
    pub layout: Option<CircuitLayout>,
}

#[derive(Debug, Default, Clone)]
pub struct CircuitLayout {
    pub x: Vec<i32>,
    pub y: Vec<i32>,
}

impl CircuitLayout {
    pub fn bounds(&self) -> Bounds {
        let (mut x_min, mut y_min) = (i32::MAX, i32::MAX);
        let (mut x_max, mut y_max) = (i32::MIN, i32::MIN);
        for x in &self.x {
            if *x > x_max {
                x_max = *x;
            }
            if *x < x_min {
                x_min = *x;
            }
        }
        for y in &self.y {
            if *y > y_max {
                y_max = *y;
            }
            if *y < y_min {
                y_min = *y;
            }
        }
        Bounds {
            x_min,
            y_min,
            x_max,
            y_max,
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
