#![allow(dead_code)]

mod node;
mod point;
mod quadtree;
mod rectangle;
mod spatial;
mod visitor;

pub(crate) enum TreeNode<T>
where
    T: Sized,
{
    Point(spatial::Spatial<T>),
    Node(Box<node::Node<T>>),
}

pub use crate::point::Point;
pub use crate::quadtree::QuadTree;
pub use crate::rectangle::Rectangle;
