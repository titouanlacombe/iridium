use nalgebra::Vector2;

pub struct UniformGravity {
    pub acceleration: Vector2<f32>,
}

impl UniformGravity {
    pub fn new(acceleration: Vector2<f32>) -> Self {
        Self { acceleration }
    }

    pub fn apply(
        &self,
        _position: &Vector2<f32>,
        _velocity: &Vector2<f32>,
        mass: &f32,
        forces: &mut Vector2<f32>,
    ) {
        *forces += self.acceleration * *mass;
    }
}

pub struct UniformDrag {
    pub coef: f32,
}

impl UniformDrag {
    pub fn new(drag: f32) -> Self {
        Self { coef: 1. - drag }
    }

    pub fn apply(
        &self,
        _position: &Vector2<f32>,
        velocity: &Vector2<f32>,
        _mass: &f32,
        forces: &mut Vector2<f32>,
    ) {
        *forces += velocity * -self.coef;
    }
}
