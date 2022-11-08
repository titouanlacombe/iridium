use nalgebra::Vector2;

pub struct QuadTree {
    pub position: Vector2<f32>,
    pub width: f32,
    pub height: f32,

    pub childs: Option<Box<[QuadTree; 4]>>,
}
