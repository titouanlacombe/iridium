use nalgebra::Vector2;

pub struct QuadTree {
    pub position: Vector2<f32>,
    pub width: f32,
    pub height: f32,

    pub child_0: Option<Box<QuadTree>>,
    pub child_1: Option<Box<QuadTree>>,
    pub child_2: Option<Box<QuadTree>>,
    pub child_3: Option<Box<QuadTree>>,
}
