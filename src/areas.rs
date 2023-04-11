use nalgebra::Vector2;

pub trait Area {
    fn contains(&self, position: Vector2<f32>) -> bool;
}

pub struct Rect {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,
}

impl Area for Rect {
    fn contains(&self, position: Vector2<f32>) -> bool {
        position.x >= self.position.x
            && position.x <= self.position.x + self.size.x
            && position.y >= self.position.y
            && position.y <= self.position.y + self.size.y
    }
}

pub struct Disk {
    pub position: Vector2<f32>,
    pub radius: f32,
}

impl Area for Disk {
    fn contains(&self, position: Vector2<f32>) -> bool {
        (position - self.position).norm() <= self.radius
    }
}

pub struct Point {
    pub position: Vector2<f32>,
}

impl Area for Point {
    fn contains(&self, position: Vector2<f32>) -> bool {
        position == self.position
    }
}
