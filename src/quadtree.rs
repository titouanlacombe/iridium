use nalgebra::Vector2;

use crate::types::{Length, Position};

pub struct QuadTree {
    pub position: Position,
    pub size: Vector2<Length>,

    pub childs: Option<Box<[QuadTree; 4]>>,
}
