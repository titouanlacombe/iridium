use nalgebra::Vector2;
use rayon::prelude::*;

use super::types::{Length, Position};

pub trait Area: Sync {
    fn contain(&self, position: Position) -> bool;

    // WARNING: indices should always be in ascending order
    fn contains(&self, positions: &Vec<Position>, indices: &mut Vec<usize>) {
        let mut result_indices: Vec<usize> = positions
            .par_iter()
            .enumerate()
            .filter_map(|(i, position)| {
                if self.contain(*position) {
                    Some(i)
                } else {
                    None
                }
            })
            .fold(
                || Vec::new(),
                |mut acc, i| {
                    acc.push(i);
                    acc
                },
            )
            .reduce(
                || Vec::new(),
                |mut acc, mut other| {
                    acc.append(&mut other);
                    acc
                },
            );

        indices.append(&mut result_indices);
    }
}

pub struct Rect {
    pub position: Position,
    pub size: Vector2<Length>,
}

impl Rect {
    pub fn new(position: Position, size: Vector2<Length>) -> Self {
        Self { position, size }
    }
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
    pub radius_squared: Length,
}

impl Disk {
    pub fn new(position: Position, radius: Length) -> Self {
        Self {
            position,
            radius_squared: radius * radius,
        }
    }
}

impl Area for Disk {
    fn contain(&self, position: Position) -> bool {
        (position - self.position).norm_squared() <= self.radius_squared
    }
}

pub struct Point {
    pub position: Position,
}

impl Point {
    pub fn new(position: Position) -> Self {
        Self { position }
    }
}

impl Area for Point {
    fn contain(&self, position: Position) -> bool {
        position == self.position
    }
}
