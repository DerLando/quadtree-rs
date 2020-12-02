use crate::{
    point::Point,
    rectangle::{Quadrant, Rectangle, RectangleRelation},
    spatial::Spatial,
    TreeNode,
};

pub(crate) struct Node<T>
where
    T: Sized,
{
    quadrants: [Option<TreeNode<T>>; 4],
    bounds: Rectangle,
}

impl<T> Node<T>
where
    T: Sized,
{
    pub(crate) const fn new_bounded(bounds: &Rectangle) -> Self {
        Self {
            quadrants: [None, None, None, None],
            bounds: *bounds,
        }
    }

    /// Gets a reference to the bounds of the node
    pub(crate) fn bounds(&self) -> &Rectangle {
        &self.bounds
    }

    pub(crate) fn quadrants(&self) -> impl Iterator<Item = &Option<TreeNode<T>>> {
        self.quadrants.iter()
    }

    pub(crate) fn quadrants_mut(&mut self) -> impl Iterator<Item = &mut Option<TreeNode<T>>> {
        self.quadrants.iter_mut()
    }

    /// Gets references to all top-level data in this node, the iterator can be empty
    pub(crate) fn data(&self) -> impl Iterator<Item = &Spatial<T>> {
        self.quadrants
            .iter()
            .filter_map(|q| q.as_ref())
            .filter_map(|tn| match tn {
                TreeNode::Node(_) => None,
                TreeNode::Point(data) => Some(data),
            })
        //.collect()
    }

    /// Gets references to all top-level nodes contained in this node, the iterator can be empty
    pub(crate) fn nodes(&self) -> impl Iterator<Item = &Node<T>> {
        self.quadrants
            .iter()
            .filter_map(|q| q.as_ref())
            .filter_map(|tn| match tn {
                TreeNode::Point(_) => None,
                TreeNode::Node(node) => Some(&**node),
            })
        //.collect()
    }

    pub(crate) fn nodes_mut(&mut self) -> impl Iterator<Item = &mut Node<T>> {
        self.quadrants
            .iter_mut()
            .filter_map(|q| q.as_mut())
            .filter_map(|tn| match tn {
                TreeNode::Point(_) => None,
                TreeNode::Node(node) => Some(&mut **node),
            })
    }

    /// Test if the given quadrant has room for an insertion
    /// this is will panic if the given quadrant has a None value, this will panic
    fn can_insert_unchecked(&self, quadrant: &Quadrant) -> bool {
        let quadrant = self
            .quadrant(quadrant)
            .as_ref()
            .expect("can_insert_unchecked can only work on Some values");
        match &quadrant {
            TreeNode::Point(_) => false,
            TreeNode::Node(_) => true,
        }
    }

    /// Try to split the specified quadrant of this node.
    /// The only case in which this actually does something,
    /// is if the given quadrant contains a single data point.
    /// The data point will be replaced with a new node in which the data point is inserted.
    fn split(&mut self, quadrant: &Quadrant) {
        // we can't split none :/
        if self.quadrant(quadrant).is_none() {
            return;
        }

        // take out whatever is in the quadrant and match on it
        match self.quadrant_mut(quadrant).take().unwrap() {
            // If we took out spatial data, we create a new node and insert the data in it
            // After that we replace the None in quadrant with the new node
            TreeNode::Point(data) => {
                let bounds = self.bounds.create_quadrant(quadrant);
                let mut node: Node<T> = Node::new_bounded(&bounds);
                node.insert(data);
                self.quadrant_mut(quadrant)
                    .replace(TreeNode::Node(Box::new(node)));
            }
            // If we took out a node, we just put it back in :)
            TreeNode::Node(n) => {
                self.quadrant_mut(quadrant).replace(TreeNode::Node(n));
            }
        }
    }

    /// Get a reference to whatever is stored at the given quadrant
    pub(crate) fn quadrant(&self, quadrant: &Quadrant) -> &Option<TreeNode<T>> {
        match quadrant {
            Quadrant::BottomLeft => &self.quadrants[0],
            Quadrant::BottomRight => &self.quadrants[1],
            Quadrant::TopRight => &self.quadrants[2],
            Quadrant::TopLeft => &self.quadrants[3],
        }
    }

    /// Get a mutable reference to whatever is stored at the given quadrant
    pub(crate) fn quadrant_mut(&mut self, quadrant: &Quadrant) -> &mut Option<TreeNode<T>> {
        match quadrant {
            Quadrant::BottomLeft => &mut self.quadrants[0],
            Quadrant::BottomRight => &mut self.quadrants[1],
            Quadrant::TopRight => &mut self.quadrants[2],
            Quadrant::TopLeft => &mut self.quadrants[3],
        }
    }

    /// Insert a [`Spatial`] into the node.
    ///
    /// # Arguments
    ///
    /// * `data` - A [`Spatial`] holding data linked to a position in space
    ///
    /// # Panics
    ///
    /// This function will blow the stack if two Spatial at the exact same position are inserted.
    ///
    ///
    pub(crate) fn insert(&mut self, data: Spatial<T>) {
        // get Quadrant of data
        let quadrant = self
            .bounds
            .find_quadrant(data.position())
            .expect("data outside of bounds!");

        // if the quadrant is still empty, we can insert the data and exit early
        if self.quadrant(&quadrant).is_none() {
            self.quadrant_mut(&quadrant).replace(TreeNode::Point(data));
            return;
        }

        // test if we can insert, or need to split
        if !self.can_insert_unchecked(&quadrant) {
            self.split(&quadrant);
        }

        // finally insert the data :)
        if let TreeNode::Node(n) = self
            .quadrant_mut(&quadrant)
            .as_mut()
            .expect("Quadrant can not be empty here")
        {
            n.insert(data)
        } else {
        }
    }

    /// Tries to remove the data at the given position,
    /// And returns ownership to it.
    /// If no data is stored at the given position, the return value will be `None`.
    /// TODO: If a node is empty after all it's contents are gone, it should be removed
    pub(crate) fn remove(&mut self, pt: &Point) -> Option<T> {
        if let Some(quadrant) = self.bounds.find_quadrant(pt) {
            match self.quadrant_mut(&quadrant).take() {
                None => None,
                Some(tn) => match tn {
                    TreeNode::Node(mut node) => {
                        let t = node.remove(pt); // assign t as we can't access node anymore after we call replace
                        self.quadrant_mut(&quadrant).replace(TreeNode::Node(node));
                        t
                    }
                    TreeNode::Point(data) => {
                        if data.position() == pt {
                            Some(data.consume())
                        } else {
                            self.quadrant_mut(&quadrant).replace(TreeNode::Point(data));
                            None
                        }
                    }
                },
            }
        } else {
            None
        }
    }

    /// Tries to find the data at the given test point.
    /// The return value will be either Some(&T), if an exact match was found, or None.
    pub(crate) fn find(&self, pt: &Point) -> Option<&T> {
        if let Some(quadrant) = self.bounds.find_quadrant(pt) {
            match self.quadrant(&quadrant) {
                None => None,
                Some(tn) => match tn {
                    TreeNode::Point(data) => {
                        if data.position() == pt {
                            Some(data.data())
                        } else {
                            None
                        }
                    }
                    TreeNode::Node(n) => n.find(pt),
                },
            }
        } else {
            None
        }
    }

    /// Finds all data stored in this node and it's child nodes,
    /// that is inside the given bounds.
    /// TODO: Visually tested, and seems to work in all edge cases :))
    pub(crate) fn find_in_bounds(&self, bounds: &Rectangle) -> Option<Vec<&Spatial<T>>> {
        let mut data: Vec<&Spatial<T>> = Vec::new();

        // get rectangle relation
        let relation = self.bounds().relation(bounds);
        match relation {
            // Disjoint is easy
            RectangleRelation::Disjoint => None,
            RectangleRelation::Intersection => {
                // first test all solo data for inclusion in query_rect
                for pt in self.data() {
                    match bounds.find_quadrant(pt.position()) {
                        None => (),
                        Some(_) => data.push(pt),
                    }
                }
                // then let the child nodes handle their points, recursively
                self.nodes().for_each(|n| match n.find_in_bounds(bounds) {
                    None => (),
                    Some(ts) => {
                        data.extend(ts);
                    }
                });
                Some(data)
            }
            RectangleRelation::Containment(query_rect_is_in_self) => {
                // if this node is fully contained in the query, we can just return all data points
                if !query_rect_is_in_self {
                    Some(self.data_children())
                } else {
                    // the query rect is fully contained in the node, find out in which quadrants

                    // first get a vec of unique quadrants the query rect corners lie in
                    let mut quadrants: Vec<Quadrant> = Vec::with_capacity(4);
                    for quadrant in bounds
                        .corners()
                        .iter()
                        .map(|c| self.bounds.find_quadrant(c))
                        .filter_map(|q| q)
                    {
                        if quadrants.contains(&quadrant) {
                        } else {
                            quadrants.push(quadrant)
                        };
                    }

                    // iterate over the unique quadrants
                    for quadrant in quadrants {
                        match self.quadrant(&quadrant) {
                            None => (),
                            Some(tn) => match tn {
                                // if we have a single data point in the same quadrant as the query_rect corner,
                                // just test for inclusion
                                TreeNode::Point(pt) => match bounds.find_quadrant(pt.position()) {
                                    Some(_) => data.push(pt),
                                    None => (),
                                },
                                // hand responsibility over to the child node
                                TreeNode::Node(node) => match node.find_in_bounds(bounds) {
                                    None => (),
                                    Some(ts) => data.extend(ts),
                                },
                            },
                        }
                    }
                    Some(data)
                }
            }
        }
    }

    pub(crate) fn minimum_coordinate_distance(&self, pt: &Point) -> Option<f32> {
        fn coordinate_distance(a: &Point, b: &Point) -> f32 {
            (a.x() - b.x()).abs().max((a.y() - b.y()).abs())
        }

        if let Some(quadrant) = self.bounds().find_quadrant(pt) {
            match self.quadrant(&quadrant) {
                None => {
                    let mut corners = self.bounds().corners();
                    corners.sort_by(|a, b| {
                        a.squared_distance(pt)
                            .partial_cmp(&b.squared_distance(pt))
                            .expect("Could not order corners")
                    });
                    corners.reverse();
                    let farthest_corner = corners.first().expect("Corners can not be empty");

                    Some(coordinate_distance(pt, &farthest_corner) + 0.1)
                }
                Some(tn) => match tn {
                    TreeNode::Point(cur_pt) => {
                        Some(coordinate_distance(pt, cur_pt.position()) + 0.1)
                    }
                    TreeNode::Node(node) => node.minimum_coordinate_distance(pt),
                },
            }
        } else {
            None
        }
    }

    /// Returns the amount of data points in the Node, maximum of 4
    /// mostly used for debug, can probably be deleted
    pub(crate) fn data_count(&self) -> u8 {
        self.data().count() as u8
    }

    /// Recursively collect references to all nodes stored in this node and all it's child nodes
    pub(crate) fn node_children(&self) -> Vec<&Node<T>> {
        let mut nodes: Vec<&Node<T>> = self.nodes().collect();

        self.nodes()
            .into_iter()
            .for_each(|n| nodes.extend(n.node_children()));

        nodes
    }

    /// Recursively collect mutable references to the nodes stored in this nodes children
    /// WARN: The direct child nodes of this node are not returned, because of
    /// a possibly double mutable borrow
    pub(crate) fn node_children_mut(&mut self) -> Vec<&mut Node<T>> {
        let mut nodes: Vec<&mut Node<T>> = Vec::new();

        self.nodes_mut()
            .into_iter()
            .for_each(|n| nodes.extend(n.node_children_mut()));

        nodes
    }

    /// Recursively collect references to all data points stored in this node and all it's child nodes.
    pub(crate) fn data_children(&self) -> Vec<&Spatial<T>> {
        let mut data: Vec<&Spatial<T>> = self.data().collect();

        self.nodes()
            .into_iter()
            .for_each(|n| data.extend(n.data_children()));

        data
    }

    /// Visit this node and all child nodes and call a parameterless closure for each visited node
    pub(crate) fn visit_nodes<F>(&self, f: &mut F)
    where
        F: FnMut(),
    {
        f(); // call function once on current node

        self.nodes().for_each(|n| n.visit_nodes(f));
    }

    /// Visit this node and all child nodes and call a closure on the currently visited node
    pub(crate) fn visit_nodes_ref<'tree, F>(&'tree self, f: &mut F)
    where
        F: FnMut(&'tree Self),
    {
        f(&self);

        self.nodes().for_each(|n| n.visit_nodes_ref(f));
    }

    /// Visit data stored in this node and all it's child nodes
    /// and call a parameterless closure for each data point.
    pub(crate) fn visit_data<F>(&self, f: &mut F)
    where
        F: FnMut(),
    {
        self.data().for_each(|_| f()); // call function on all data points
        self.nodes().for_each(|n| n.visit_data(f)); // pass function recursively to all child nodes
    }

    /// Visit data stored in this node and all it's child nodes
    /// and call a closure on each data point
    pub(crate) fn visit_data_ref<F>(&self, f: &mut F)
    where
        F: FnMut(&Spatial<T>),
    {
        self.data().for_each(|data| f(data));
        self.nodes().for_each(|n| n.visit_data_ref(f));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.quadrants.iter().filter_map(|q| q.as_ref()).count() == 0
    }

    /// Collapse this node recursively, returning all data points stored
    pub(crate) fn collapse(&mut self) -> Vec<Spatial<T>> {
        let mut data: Vec<Spatial<T>> = Vec::new();

        // first all child nodes
        self.node_children_mut().iter_mut().for_each(|n| {
            n.quadrants_mut().for_each(|q| match q.take() {
                None => (),
                Some(tn) => match tn {
                    TreeNode::Point(pt) => data.push(pt),
                    TreeNode::Node(mut node) => data.extend(node.collapse()),
                },
            })
        });

        // // then we violate DRY and do it again for top-level quadrants :/
        self.quadrants_mut().for_each(|q| match q.take() {
            None => (),
            Some(tn) => match tn {
                TreeNode::Point(pt) => data.push(pt),
                TreeNode::Node(mut node) => data.extend(node.collapse()),
            },
        });

        data
    }

    // this only removes the child at the bottom of the stack
    // TODO: We probably need space queries to merge nodes that contain points after removal
    pub(crate) fn shrink_children(&mut self) {
        self.quadrants.iter_mut().for_each(|q| match q.take() {
            None => (),
            Some(tn) => match tn {
                TreeNode::Point(_) => (),
                TreeNode::Node(mut node) => {
                    if node.is_empty() {
                    } else {
                        node.shrink_children();
                        q.replace(TreeNode::Node(node));
                    }
                }
            },
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn node_can_insert() {
        // Arrange
        let mut node: Node<u8> = Node::new_bounded(&Rectangle::new((0.0, 0.0), 10.0, 20.0));
        assert_eq!(0, node.data_count());

        node.insert((0, Point::new(0.1, 0.2)).into());
        assert_eq!(1, node.data_count()); // We have one data point in bl corner

        node.insert((12, Point::new(0.1, 0.3)).into());
        assert_eq!(0, node.data_count()); // bl corner is split, so we have 0 data points again

        node.insert((4, Point::new(6.0, 0.5)).into());
        assert_eq!(1, node.data_count()); // br corner has a new data point, count should be 1

        node.insert((4, Point::new(6.0, 12.0)).into());
        assert_eq!(2, node.data_count()); // tr corner has a new data point, count should be 2

        node.insert((4, Point::new(2.0, 18.0)).into());
        assert_eq!(3, node.data_count()); // br corner has a new data point, count should be 3
    }

    #[test]
    fn node_can_remove() {
        // Arrange
        let mut node: Node<u8> = Node::new_bounded(&Rectangle::new((0.0, 0.0), 10.0, 20.0));
        assert_eq!(0, node.data_count());
        assert!(node.is_empty());

        node.insert((0, Point::new(0.1, 0.2)).into());
        assert_eq!(1, node.data_count()); // We have one data point in bl corner

        assert_eq!(Some(0), node.remove(&Point::new(0.1, 0.2)));

        // insert some data points close to each other to trigger some splits
        node.insert((0, (0.0, 0.0)).into());
        node.insert((0, (0.0, 0.1)).into());
        node.insert((0, (0.0, 0.2)).into());

        assert_eq!(7, node.node_children().len());

        // remove data points again
        node.remove(&(0.0, 0.0).into());
        node.remove(&(0.0, 0.1).into());
        node.remove(&(0.0, 0.2).into());

        // child_node count should  not change, but now they are all empty
        assert_eq!(7, node.node_children().len());

        // collapse the node
        assert_eq!(0, node.collapse().len());

        // now the node should have no childs anymore
        assert_eq!(0, node.node_children().len());
    }

    #[test]
    fn node_can_query_by_bounds() {
        // Arrange
        let mut node: Node<usize> = Node::new_bounded(&Rectangle::new((0.0, 0.0), 10.0, 10.0));
        assert_eq!(0, node.data_count());
        assert!(node.is_empty());

        let mut points: Vec<Point> = Vec::with_capacity(100);
        for i in 0..10 {
            for j in 0..10 {
                points.push(Point::new(i as f32 + 0.5, j as f32 + 0.5));
            }
        }

        for (index, point) in points.iter().enumerate() {
            node.insert((index, *point).into());

            println!("{:?}: {:?}", index, point);
        }

        let query_rect = Rectangle::new((5.0, 5.0), 5.6, 5.6);
        println!("{:?}", node.find_in_bounds(&query_rect));
        assert_eq!(25, node.find_in_bounds(&query_rect).unwrap().len())
    }

    #[test]
    fn node_size() {
        println!("size of u8: {:?}", std::mem::size_of::<u8>());
        println!("size of f32: {:?}", std::mem::size_of::<f32>());
        println!("size of Point: {:?}", std::mem::size_of::<Point>());
        println!("size of Rectangle: {:?}", std::mem::size_of::<Rectangle>());
        println!(
            "size of Spatial<u8>: {:?}",
            std::mem::size_of::<Spatial<u8>>()
        );
        println!(
            "size of Box<Node<u8>>: {:?}",
            std::mem::size_of::<Box<Node<u8>>>()
        );
        println!(
            "size of TreeNode<u8>: {:?}",
            std::mem::size_of::<TreeNode<u8>>()
        );
        println!("size of Node<u8>: {:?}", std::mem::size_of::<Node<u8>>());

        assert!(true)
    }
}
