use crate::{node::Node, rectangle::Quadrant, spatial::Spatial, TreeNode};

pub(crate) trait Visitor<T>
where
    T: Sized,
{
    fn visit_quadrant(&mut self, quadrant: &Option<TreeNode<T>>) {
        walk_quadrant(self, quadrant);
    }

    fn visit_tree_node(&mut self, tree_node: &TreeNode<T>) {
        walk_tree_node(self, tree_node);
    }

    fn visit_node(&mut self, node: &Node<T>) {
        walk_node(self, node);
    }

    fn visit_point(&mut self, data: &Spatial<T>) {
        walk_point(self, data);
    }
}

pub(crate) fn walk_quadrant<T, V: Visitor<T> + ?Sized>(
    visitor: &mut V,
    quadrant: &Option<TreeNode<T>>,
) {
    match quadrant {
        Some(tn) => visitor.visit_tree_node(tn),
        None => (),
    }
}

pub(crate) fn walk_tree_node<T, V: Visitor<T> + ?Sized>(visitor: &mut V, tree_node: &TreeNode<T>) {
    match tree_node {
        TreeNode::Point(pt) => visitor.visit_point(pt),
        TreeNode::Node(n) => visitor.visit_node(n),
    }
}

pub(crate) fn walk_node<T, V: Visitor<T> + ?Sized>(visitor: &mut V, node: &Node<T>) {
    visitor.visit_quadrant(node.quadrant(&Quadrant::BottomLeft));
    visitor.visit_quadrant(node.quadrant(&Quadrant::BottomRight));
    visitor.visit_quadrant(node.quadrant(&Quadrant::TopRight));
    visitor.visit_quadrant(node.quadrant(&Quadrant::TopLeft));
}

pub(crate) fn walk_point<T, V: Visitor<T> + ?Sized>(_visitor: &mut V, _point: &Spatial<T>) {}

pub(crate) struct TwoWayVisitor<'qt, T>
where
    T: Sized,
{
    parent: &'qt Node<T>,
}

impl<'qt, T> Visitor<T> for TwoWayVisitor<'qt, T>
where
    T: Sized,
{
    fn visit_quadrant(&mut self, quadrant: &Option<TreeNode<T>>) {
        walk_quadrant(self, quadrant);
    }

    fn visit_tree_node(&mut self, tree_node: &TreeNode<T>) {
        walk_tree_node(self, tree_node);
    }

    fn visit_node(&mut self, node: &Node<T>) {
        walk_node(self, node);
    }

    fn visit_point(&mut self, data: &Spatial<T>) {
        walk_point(self, data);
    }
}

impl<'qt, T> TwoWayVisitor<'qt, T>
where
    T: Sized,
{
    fn visit_parent(&mut self) {
        walk_node(self, self.parent)
    }
}
