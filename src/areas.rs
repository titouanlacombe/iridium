use nalgebra::Vector2;

pub trait Area {
    fn contain(&self, position: Vector2<f32>) -> bool;

    fn contains(&self, positions: &Vec<Vector2<f32>>, indices: &mut Vec<usize>) {
        for (i, position) in positions.iter().enumerate() {
            if self.contain(*position) {
                indices.push(i);
            }
        }
    }
}

pub struct Rect {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,
}

impl Area for Rect {
    fn contain(&self, position: Vector2<f32>) -> bool {
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
    fn contain(&self, position: Vector2<f32>) -> bool {
        (position - self.position).norm_squared() <= self.radius * self.radius
    }
}

pub struct Point {
    pub position: Vector2<f32>,
}

impl Area for Point {
    fn contain(&self, position: Vector2<f32>) -> bool {
        position == self.position
    }
}
