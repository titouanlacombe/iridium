use nalgebra::Vector2;

pub trait Area {
    fn contains(&self, position: Vector2<f32>) -> bool;
    fn rand(&self) -> Vector2<f32>;
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

    fn rand(&self) -> Vector2<f32> {
        Vector2::new(
            self.position.x + rand::random::<f32>() * self.size.x,
            self.position.y + rand::random::<f32>() * self.size.y,
        )
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

    fn rand(&self) -> Vector2<f32> {
        let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;

        // For a uniform distribution, we need to square root the random number
        let radius = rand::random::<f32>().sqrt() * self.radius;
        self.position + Vector2::new(radius * angle.cos(), radius * angle.sin())
    }
}

pub struct Point {
    pub position: Vector2<f32>,
}

impl Area for Point {
    fn contains(&self, position: Vector2<f32>) -> bool {
        position == self.position
    }

    fn rand(&self) -> Vector2<f32> {
        self.position
    }
}
