use crate::point::Point;

/// A rectangle representation, anchored at the bottom left corner
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rectangle {
    /// anchor of rectangle at bottom left corner
    anchor: Point,

    width: f32,
    height: f32,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Quadrant {
    BottomLeft,
    BottomRight,
    TopRight,
    TopLeft,
}

#[derive(Debug, PartialEq)]
pub(crate) enum RectangleRelation {
    /// Both rectangles are disjoint from another
    Disjoint,

    /// The Rectangles are Intersecting at some points
    Intersection,

    /// One rectangle fully contains the other, as hinted by the flag
    /// if the inner value is true, then A(B), if it is false then B(A)
    Containment(bool),
}

impl Rectangle {
    /// Create a new [`Rectangle`] struct
    ///
    /// # Arguments
    ///
    /// * `anchor` - The anchor of the Rectangle, located at the bottom left corner
    /// * `width` - The width of the rectangle
    /// * `height` - The height of the rectangle
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{Rectangle, Point};
    /// let rect = Rectangle::new((0.0, 0.0), 10.0, 20.0);
    /// assert_eq!(10.0, rect.max_x());
    /// assert_eq!(20.0, rect.max_y());
    /// ```
    pub fn new(anchor: impl Into<Point>, width: f32, height: f32) -> Self {
        Self {
            anchor: anchor.into(),
            width,
            height,
        }
    }

    /// Creates a new rectangle centered around the given anchor
    ///
    /// # Arguments
    ///
    /// * `center` - The center point of the rectangle
    /// * `width` - The width of the rectangle
    /// * `height` - The height of the rectangle
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{Rectangle, Point};
    /// let rect = Rectangle::new_centered((0.0, 0.0), 10.0, 20.0);
    /// assert_eq!(5.0, rect.max_x());
    /// assert_eq!(10.0, rect.max_y());
    /// ```
    pub fn new_centered(center: impl Into<Point>, width: f32, height: f32) -> Self {
        let pt = center.into();
        Self {
            anchor: Point::new(pt.x() - width / 2.0, pt.y() - height / 2.0),
            width,
            height,
        }
    }

    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn center(&self) -> Point {
        Point::new(
            self.min_x() + self.width / 2.0,
            self.min_y() + self.height / 2.0,
        )
    }

    pub fn min_x(&self) -> f32 {
        self.anchor.x()
    }
    pub fn max_x(&self) -> f32 {
        self.anchor.x() + self.width
    }
    pub fn min_y(&self) -> f32 {
        self.anchor.y()
    }
    pub fn max_y(&self) -> f32 {
        self.anchor.y() + self.height
    }

    /// # Quadrant definitions
    /// Quadrants are defined from strongest to weakest, from bottom left to top right
    ///
    /// |----|:----|
    /// |    |:    |
    /// |....|:....|
    /// |----|:----|
    /// |    |:    |
    /// x----|:----|
    ///
    pub(crate) fn find_quadrant(&self, pt: &Point) -> Option<Quadrant> {
        let min_x = self.min_x();
        let min_y = self.min_y();
        let max_x = self.max_x();
        let max_y = self.max_y();

        // Test if the point is inside the general bounds of the rectangle
        if (pt.x() < min_x) || (pt.x() > max_x) || (pt.y() < min_y) || (pt.y() > max_y) {
            None
        } else {
            let mid_x = (min_x + max_x) / 2.0;
            let mid_y = (min_y + max_y) / 2.0;

            if pt.x() > mid_x {
                if pt.y() > mid_y {
                    Some(Quadrant::TopRight)
                } else {
                    Some(Quadrant::BottomRight)
                }
            } else {
                if pt.y() > mid_y {
                    Some(Quadrant::TopLeft)
                } else {
                    Some(Quadrant::BottomLeft)
                }
            }
        }
    }

    pub(crate) fn create_quadrant(&self, quadrant: &Quadrant) -> Self {
        let width = self.width / 2.0;
        let height = self.height / 2.0;
        let anchor: Point;

        match quadrant {
            Quadrant::BottomLeft => anchor = self.anchor.clone(),
            Quadrant::BottomRight => anchor = Point::new(self.anchor.x() + width, self.anchor.y()),
            Quadrant::TopRight => {
                anchor = Point::new(self.anchor.x() + width, self.anchor.y() + height)
            }
            Quadrant::TopLeft => anchor = Point::new(self.anchor.x(), self.anchor.y() + height),
        }

        Self {
            anchor,
            width,
            height,
        }
    }

    pub(crate) fn corners(&self) -> [Point; 4] {
        [
            (self.min_x(), self.min_y()).into(),
            (self.max_x(), self.min_y()).into(),
            (self.max_x(), self.max_y()).into(),
            (self.min_x(), self.max_y()).into(),
        ]
    }

    pub(crate) fn relation(&self, other: &Rectangle) -> RectangleRelation {
        let inner_quadrants: Vec<Option<Quadrant>> = other
            .corners()
            .iter()
            .map(|c| self.find_quadrant(c))
            .collect();

        if inner_quadrants.iter().all(|q| q.is_none()) {
            // all corners are outside of this rectangle,
            // do a complicated lookup to test for intersections,
            match self
                .corners()
                .iter()
                .map(|c| other.find_quadrant(c))
                .filter_map(|q| q)
                .count()
            {
                // no corners from self inside of other
                0 => RectangleRelation::Disjoint,

                // some corners inside of other
                1 | 2 | 3 => RectangleRelation::Intersection,

                // fully contained in other
                4 => RectangleRelation::Containment(false),

                // if we ever get more than 4 corners here, we SHOULD panic
                _ => unreachable!(),
            }
        } else {
            // some or all corners of other are inside of this rectangle,
            // so it's either fully contained, or intersecting
            if inner_quadrants.iter().all(|q| q.is_some()) {
                RectangleRelation::Containment(true)
            } else {
                RectangleRelation::Intersection
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rectangle_should_find_quadrants() {
        // Arrange
        let anchor = Point::new(0.0, 0.0);
        let rect = Rectangle::new(anchor, 2.0, 3.0);
        let center = Point::new(1.0, 1.5);

        // Assert
        assert_eq!(Quadrant::BottomLeft, rect.find_quadrant(&center).unwrap());
        assert_eq!(Quadrant::BottomLeft, rect.find_quadrant(&anchor).unwrap());
        assert!(rect.find_quadrant(&Point::new(-1.0, 0.0)).is_none());

        assert_eq!(
            Quadrant::BottomRight,
            rect.find_quadrant(&Point::new(1.23, 1.5)).unwrap()
        );
        assert_eq!(
            Quadrant::TopLeft,
            rect.find_quadrant(&Point::new(1.0, 1.89)).unwrap()
        );
    }

    #[test]
    fn rectangle_should_create_valid_quadrants() {
        // Arrange
        let rect = Rectangle::new(Point::new(0.0, 0.0), 1.0, 3.5);
        let bl_rect = Rectangle::new(Point::new(0.0, 0.0), 0.5, 1.75);
        let br_rect = Rectangle::new(Point::new(0.5, 0.0), 0.5, 1.75);
        let tr_rect = Rectangle::new(Point::new(0.5, 1.75), 0.5, 1.75);
        let tl_rect = Rectangle::new(Point::new(0.0, 1.75), 0.5, 1.75);

        // Assert
        assert_eq!(bl_rect, rect.create_quadrant(&Quadrant::BottomLeft));
        assert_eq!(br_rect, rect.create_quadrant(&Quadrant::BottomRight));
        assert_eq!(tr_rect, rect.create_quadrant(&Quadrant::TopRight));
        assert_eq!(tl_rect, rect.create_quadrant(&Quadrant::TopLeft));
    }

    #[test]
    fn rectangle_should_find_correct_relations() {
        // Arrange
        let rect = Rectangle::new(Point::new(0.0, 0.0), 5.0, 5.0);

        let other = Rectangle::new(Point::new(1.0, 1.0), 3.0, 3.0);
        assert_eq!(RectangleRelation::Containment(true), rect.relation(&other));

        let other = Rectangle::new(Point::new(-1.0, -1.0), 6.5, 8.5);
        assert_eq!(RectangleRelation::Containment(false), rect.relation(&other));

        let other = Rectangle::new(Point::new(-5.0, -10.0), 6.5, 8.5);
        assert_eq!(RectangleRelation::Disjoint, rect.relation(&other));

        let other = Rectangle::new(Point::new(-1.0, -1.0), 3.0, 3.0);
        assert_eq!(RectangleRelation::Intersection, rect.relation(&other));

        let other = Rectangle::new(Point::new(-1.0, -1.0), 8.0, 3.0);
        assert_eq!(RectangleRelation::Intersection, rect.relation(&other));
    }
}
