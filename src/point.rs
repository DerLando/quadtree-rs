#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn squared_distance(&self, other: &Point) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}

impl From<(f32, f32)> for Point {
    fn from(args: (f32, f32)) -> Self {
        Self::new(args.0, args.1)
    }
}
