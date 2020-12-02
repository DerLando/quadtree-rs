use crate::{node::Node, point, point::Point, rectangle::Rectangle, spatial::Spatial};

/// # QuadTree
/// A simple, naive implementation of a basic `QuadTree` data structure.
/// Allows insertion of metadata which implements [`Sized`] with
/// a position in 2d-Space, a [`Point`].
pub struct QuadTree<T>
where
    T: Sized,
{
    root: Node<T>,
    bounds: Rectangle,
}

impl<T> QuadTree<T>
where
    T: Sized,
{
    /// Creates a `QuadTree` with the given bounds, over generic, `Sized` data `T`
    ///
    /// # Arguments
    ///
    /// * `bounds` - A [`Rectangle`] by which the quadtree will be bounded
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let quadtree_u8: QuadTree<u8> = QuadTree::new_bounded(&bounds);
    /// let quadtree_bool: QuadTree<bool> = QuadTree::new_bounded(&bounds);
    /// ```
    ///
    pub const fn new_bounded(bounds: &Rectangle) -> Self {
        Self {
            root: Node::new_bounded(bounds),
            bounds: *bounds,
        }
    }

    /// Tries to insert a given spatial data into the quadtree,
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be inserted, has to implement `Into<Spatial<T>>`
    ///
    /// # Failure
    ///
    /// This function can fail and will return `false` if it did.
    ///
    /// Fail cases are:
    /// * The spatial data is outside of the bounds of the tree
    /// * The tree already contains a data point at the given position
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// // insert succeeds
    /// assert!(quadtree.insert(12u8, (0.0, 0.5)));
    ///
    /// // insert fails if we try to insert more data to the same position
    /// assert!(!quadtree.insert(6u8, (0.0, 0.5)));
    ///
    /// // insert fails because out of bounds of tree
    /// assert!(!quadtree.insert(2u8, (-2.0, 5.0)));
    /// ```
    ///
    pub fn insert(&mut self, data: T, position: impl Into<Point>) -> bool {
        let data: Spatial<T> = (data, position.into()).into();

        // test if data is in tree bounds
        if self.bounds.find_quadrant(data.position()).is_none() {
            return false;
        }

        // test if data point is already contained, this would otherwise blow the stack
        if self.contains(*data.position()) {
            return false;
        }

        // finally insert
        self.root.insert(data);
        true
    }

    /// Inserts data into the tree without any checks.
    /// If you know your data to be valid, this is faster then [`QuadTree::insert`]
    pub fn insert_unchecked(&mut self, data: T, position: impl Into<Point>) {
        // Assume the user knows what he is doing :/
        self.root.insert((data, position.into()).into())
    }

    fn insert_unchecked_spatial(&mut self, spatial: Spatial<T>) {
        self.root.insert(spatial)
    }

    /// Removes the data stored at the given Point, giving back ownership to `T`
    ///
    /// # Arguments
    ///
    /// * `pt` - The [`Point`] at which to remove data, or anything implementing `Into<Point>`
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// assert!(quadtree.insert(12u8, (0.0, 0.5)));
    ///
    /// assert_eq!(Some(12), quadtree.remove((0.0, 0.5)));
    /// assert_eq!(None, quadtree.remove((0.0, 0.5)));
    /// ```
    ///
    pub fn remove(&mut self, pt: impl Into<Point>) -> Option<T> {
        self.root.remove(&pt.into())
    }

    /// Replace data in the `QuadTree` with other data
    ///
    /// # Arguments
    ///
    /// * `data` - The data to replace with
    /// * `position` - The [`Point`] at which to replace data, or anything implementing `Into<Point>`
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// assert!(quadtree.insert(12u8, (0.0, 0.5)));
    /// assert_eq!(Some(12u8), quadtree.replace(6, (0.0, 0.5)));
    /// assert_eq!(Some(6), quadtree.replace(4, (0.0, 0.5)));
    /// assert_eq!(Some(4), quadtree.replace(12, (0.0, 0.5)));
    ///
    pub fn replace(&mut self, data: T, position: impl Into<Point>) -> Option<T> {
        let pt = position.into();
        match self.remove(pt) {
            None => None,
            Some(t) => {
                self.insert(data, pt);
                Some(t)
            }
        }
    }

    /// Shrink the tree, to remove unused nodes left after removal operations
    /// Wow what a hack ...
    pub fn shrink(&mut self) {
        for data in self.root.collapse() {
            self.insert_unchecked_spatial(data);
        }
    }

    /// Test if the `QuadTree` already contains data at the given position.
    ///
    /// # Arguments
    ///
    /// * `pt` - The point for which to test, or anything implementing `Into<Point>`
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// quadtree.insert(3u8, (2.0, 7.0));
    /// assert!(quadtree.contains((2.0, 7.0)));
    /// ```
    ///
    pub fn contains(&self, pt: impl Into<Point>) -> bool {
        match self.root.find(&pt.into()) {
            None => false,
            Some(_) => true,
        }
    }

    /// Test if the `QuadTree` already contains data at the given position and returns a reference to it
    ///
    /// # Arguments
    ///
    /// * `pt` - The point for which to test, or anything implementing `Into<Point>`
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// quadtree.insert(3u8, (2.0, 7.0));
    /// assert_eq!(Some(&3u8), quadtree.find_exact((2.0, 7.0)));
    /// assert_eq!(None, quadtree.find_exact((-5.0, 12.0)));
    /// assert_eq!(None, quadtree.find_exact((2.001, 6.999)));
    /// ```
    ///
    pub fn find_exact(&self, pt: impl Into<Point>) -> Option<&T> {
        match self.root.find(&pt.into()) {
            None => None,
            Some(t) => Some(t),
        }
    }

    // TODO: pricate nearest neighbour that returns an iterator over spatial<T>
    // we can then consume this in the public nearest neighbour methods and map to either T or position
    fn find_nearest_neighbors(
        &self,
        pt: &Point,
        n: usize,
    ) -> Option<impl Iterator<Item = &Spatial<T>>> {
        if let Some(dist) = self.root.minimum_coordinate_distance(&pt) {
            let query_rect = Rectangle::new_centered(*pt, dist * 2.0, dist * 2.0);
            match self.root.find_in_bounds(&query_rect) {
                Some(mut data) => {
                    data.sort_by(|a, b| {
                        a.position()
                            .squared_distance(&pt)
                            .partial_cmp(&b.position().squared_distance(&pt))
                            .expect("Failed to order candidates!")
                    });
                    Some(data.into_iter().take(n))
                }
                None => None,
            }
        } else {
            None
        }
    }

    /// Finds the data of the nearest neighbor to a given test point, inside the quadtree.
    ///
    /// # Arguments
    ///
    /// * `pt` - The point for which to test, or anything implementing `Into<Point>`
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// quadtree.insert(3u8, (2.0, 7.0));
    /// quadtree.insert(2, (2.5, 7.5));
    ///
    /// assert_eq!(Some(&2u8), quadtree.find_nearest_neighbor((3.0, 8.0)));
    ///
    /// ```
    pub fn find_nearest_neighbor(&self, pt: impl Into<Point>) -> Option<&T> {
        let pt = pt.into();
        if let Some(mut iter) = self.find_nearest_neighbors(&pt, 1) {
            if let Some(spatial) = iter.next() {
                Some(spatial.data())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Finds the position of the nearest neighbor to a given test point, inside the quadtree.
    ///
    /// # Arguments
    ///
    /// * `pt` - The point for which to test, or anything implementing `Into<Point>`
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// quadtree.insert(3u8, (2.0, 7.0));
    /// quadtree.insert(2, (2.5, 7.5));
    ///
    /// assert_eq!(Some((2.5, 7.5).into()), quadtree.find_nearest_neighbor_position((3.0, 8.0)));
    ///
    /// ```
    pub fn find_nearest_neighbor_position(&self, pt: impl Into<Point>) -> Option<Point> {
        let pt = pt.into();
        if let Some(mut iter) = self.find_nearest_neighbors(&pt, 1) {
            if let Some(spatial) = iter.next() {
                Some(*spatial.position())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Finds all data contained in the given rectangle bounds
    ///
    /// # Arguments
    ///
    /// * `bounds` - The bounds inside of which data should be returned
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// quadtree.insert(3u8, (2.0, 7.0));
    /// quadtree.insert(2, (2.5, 7.5));
    /// quadtree.insert(1, (1.0, 5.5));
    ///
    /// let bounds = Rectangle::new((2.0, 6.0), 4.0, 5.0);
    /// assert_eq!(Some(vec![&2u8, &3]), quadtree.find_in_bounds(&bounds));
    ///
    /// ```
    pub fn find_in_bounds(&self, bounds: &Rectangle) -> Option<Vec<&T>> {
        if let Some(data) = self.root.find_in_bounds(bounds) {
            Some(data.iter().map(|d| d.data()).collect())
        } else {
            None
        }
    }

    /// Finds all positions contained in the given rectangle bounds
    ///
    /// # Arguments
    ///
    /// * `bounds` - The bounds inside of which data should be returned
    ///
    /// # Examples
    ///
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// quadtree.insert(3u8, (2.0, 7.0));
    /// quadtree.insert(2, (2.5, 7.5));
    /// quadtree.insert(1, (1.0, 5.5));
    ///
    /// let bounds = Rectangle::new((2.0, 6.0), 4.0, 5.0);
    /// assert_eq!(Some(vec![(2.5, 7.5).into(), (2.0, 7.0).into()]), quadtree.find_in_bounds_positions(&bounds));
    ///
    /// ```
    pub fn find_in_bounds_positions(&self, bounds: &Rectangle) -> Option<Vec<Point>> {
        if let Some(data) = self.root.find_in_bounds(bounds) {
            Some(data.iter().map(|d| *d.position()).collect())
        } else {
            None
        }
    }

    /// Calculate the number of all nodes in the tree
    ///
    /// # Examples
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// // node count on empty tree is 1, for the root
    /// assert_eq!(1, quadtree.node_count());
    ///
    /// // insert two points close to each other, the root node has to split
    /// quadtree.insert(3u8, (2.0, 7.0));
    /// quadtree.insert(2, (2.5, 7.5));
    ///
    /// assert_eq!(4, quadtree.node_count());
    /// ```
    pub fn node_count(&self) -> usize {
        let mut count: usize = 0;
        let mut add_node_count_to_x = || count += 1;
        self.root.visit_nodes(&mut add_node_count_to_x);

        count
    }

    /// Get references to the bounds of all nodes in this tree
    /// The bounds are stored as **a**xis **a**ligned **b**ounding **b**oxes.
    ///
    /// # Examples
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// // insert two points close to each other, the root node has to split
    /// quadtree.insert(3u8, (2.0, 7.0));
    /// quadtree.insert(2, (2.5, 7.5));
    ///
    /// let bounds = [
    ///     Rectangle::new((0.0, 0.0), 5.0, 10.0), // root bounds
    ///     Rectangle::new((0.0, 5.0), 2.5, 5.0), // root -> top left
    ///     Rectangle::new((1.25, 5.0), 1.25, 2.5),  // root -> top left -> top left
    ///     Rectangle::new((1.875, 6.25), 0.625, 1.25), // root -> top left -> top left -> top right
    /// ];
    /// assert_eq!(bounds.iter().map(|b| b).collect::<Vec<&Rectangle>>(), quadtree.aabbs());
    /// ```
    ///
    ///
    pub fn aabbs<'tree>(&'tree self) -> Vec<&'tree Rectangle> {
        let mut aabbs: Vec<&Rectangle> = Vec::new();
        let mut push_node_bounds_onto_x = |node: &'tree Node<T>| aabbs.push(node.bounds());
        self.root.visit_nodes_ref(&mut push_node_bounds_onto_x);

        aabbs
    }

    /// Get the number of all data points *(or leaf nodes)* stored in this tree
    ///
    /// # Examples
    /// ```
    /// use quadtree::{QuadTree, Rectangle};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// // len of empty tree is 0
    /// assert_eq!(0, quadtree.len());
    ///
    /// quadtree.insert(3u8, (2.0, 7.0));
    /// assert_eq!(1, quadtree.len());
    ///
    /// quadtree.insert(2, (2.5, 7.5));
    /// assert_eq!(2, quadtree.len());
    ///
    /// quadtree.remove((2.5, 7.5));
    /// assert_eq!(1, quadtree.len());
    /// ```
    ///
    pub fn len(&self) -> usize {
        let mut len: usize = 0;
        let mut add_data_count_to_x = |node: &Node<T>| len += node.data().count();
        self.root.visit_nodes_ref(&mut add_data_count_to_x);

        len
    }

    /// Returns an iterator over the positions of all data points in this tree
    ///
    /// # Examples
    /// ```
    /// use quadtree::{QuadTree, Rectangle, Point};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// quadtree.insert(3u8, (2.0, 7.0));
    /// quadtree.insert(2, (2.5, 7.5));
    ///
    /// let mut positions = quadtree.iter_positions();
    /// assert_eq!(&Point::new(2.5, 7.5), positions.next().unwrap());
    /// assert_eq!(&Point::new(2.0, 7.0), positions.next().unwrap());
    /// assert!(positions.next().is_none());
    /// ```
    pub fn iter_positions(&self) -> impl Iterator<Item = &Point> {
        self.root
            .data_children()
            .into_iter()
            .map(|spatial| spatial.position())
    }

    /// Returns an iterator over the data of all data points in this tree
    ///
    /// # Examples
    /// ```
    /// use quadtree::{QuadTree, Rectangle, Point};
    /// let bounds = Rectangle::new((0.0, 0.0), 5.0, 10.0);
    /// let mut quadtree = QuadTree::new_bounded(&bounds);
    ///
    /// quadtree.insert(3u8, (2.0, 7.0));
    /// quadtree.insert(2, (2.5, 7.5));
    ///
    /// let mut positions = quadtree.iter_data();
    /// assert_eq!(&2, positions.next().unwrap());
    /// assert_eq!(&3, positions.next().unwrap());
    /// assert!(positions.next().is_none());
    /// ```
    pub fn iter_data(&self) -> impl Iterator<Item = &T> {
        self.root
            .data_children()
            .into_iter()
            .map(|spatial| spatial.data())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn quadtree_can_insert() {
        // Arrange
        let bounds: Rectangle = Rectangle::new((0.0, 0.0), 10.0, 20.0);
        let mut quadtree: QuadTree<u8> = QuadTree::new_bounded(&bounds);

        // insert a data point unchecked
        quadtree.insert_unchecked(4, (0.0, 0.5));

        // assert it's where we assume it to be
        assert!(quadtree.contains((0.0, 0.5)));
        assert_eq!(Some(&4u8), quadtree.find_exact((0.0, 0.5)));

        quadtree.insert(2, (1.0, 1.0));
        quadtree.insert(2, (2.0, 1.0));
        quadtree.insert(2, (0.5, 1.0));

        assert_eq!(5, quadtree.node_count());
        assert_eq!(4, quadtree.len());
    }
}
