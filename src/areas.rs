use nalgebra::Vector2;

use crate::types::{Length, Position};

pub trait Area {
    fn contain(&self, position: Position) -> bool;

    // WARNING: indices should always be in ascending order
    fn contains(&self, positions: &Vec<Position>, indices: &mut Vec<usize>) {
        for (i, position) in positions.iter().enumerate() {
            if self.contain(*position) {
                indices.push(i);
            }
        }
    }
}

pub struct Rect {
    pub position: Position,
    pub size: Vector2<Length>,
}

impl Area for Rect {
    fn contain(&self, position: Position) -> bool {
        position.x >= self.position.x
            && position.x <= self.position.x + self.size.x
            && position.y >= self.position.y
            && position.y <= self.position.y + self.size.y
    }
}

pub struct Disk {
    pub position: Position,
    pub radius: Length,
}

impl Area for Disk {
    fn contain(&self, position: Position) -> bool {
        (position - self.position).norm_squared() <= self.radius * self.radius
    }
}

pub struct Point {
    pub position: Position,
}

impl Area for Point {
    fn contain(&self, position: Position) -> bool {
        position == self.position
    }
}
