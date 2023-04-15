use nalgebra::Vector2;

pub trait Area {
    fn contains(&self, positions: &Vec<Vector2<f32>>, indices: &mut Vec<usize>);
}

pub struct Rect {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,
}

impl Area for Rect {
    fn contains(&self, positions: &Vec<Vector2<f32>>, indices: &mut Vec<usize>) {
        for (i, position) in positions.iter().enumerate() {
            if position.x >= self.position.x
                && position.x <= self.position.x + self.size.x
                && position.y >= self.position.y
                && position.y <= self.position.y + self.size.y
            {
                indices.push(i);
            }
        }
    }
}

pub struct Disk {
    pub position: Vector2<f32>,
    pub radius: f32,
}

impl Area for Disk {
    fn contains(&self, positions: &Vec<Vector2<f32>>, indices: &mut Vec<usize>) {
        let r_squared = self.radius * self.radius;
        for (i, position) in positions.iter().enumerate() {
            if (position - self.position).norm_squared() <= r_squared {
                indices.push(i);
            }
        }
    }
}

pub struct Point {
    pub position: Vector2<f32>,
}

impl Area for Point {
    fn contains(&self, positions: &Vec<Vector2<f32>>, indices: &mut Vec<usize>) {
        for (i, position) in positions.iter().enumerate() {
            if position == &self.position {
                indices.push(i);
            }
        }
    }
}
