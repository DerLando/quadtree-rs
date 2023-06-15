/// A simple 2-dimensional point struct.
/// Implemented here to don't have any external dependencies.
/// In the future there should probably a feature to replace this
/// With a widely used point type from the rust eco system...
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    /// Create a new [`Point`] from it's x and y components
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Read back the inner x component of the [`Point`]
    pub fn x(&self) -> f32 {
        self.x
    }
    /// Read back the inner y component of the [`Point`]
    pub fn y(&self) -> f32 {
        self.y
    }

    pub(crate) fn squared_distance(&self, other: &Point) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}

impl From<(f32, f32)> for Point {
    fn from(args: (f32, f32)) -> Self {
        Self::new(args.0, args.1)
    }
}
