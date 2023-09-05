use nalgebra::Vector2;
use sfml::system::Vector2f;

use crate::types::{Position, Scalar};

pub trait CoordinateSystem {
    fn sim2screen(&self, position: Position) -> Vector2f;
    fn screen2sim(&self, position: Vector2f) -> Position;
    fn set_screen_size(&mut self, screen_size: Vector2<u32>);
    fn set_sim_size(&mut self, sim_size: Vector2<Scalar>);
}

pub struct FlippedCoordinateSystem {
    screen_size: Vector2<u32>,
}

impl FlippedCoordinateSystem {
    pub fn new(screen_size: Vector2<u32>) -> Self {
        Self { screen_size }
    }
}

impl CoordinateSystem for FlippedCoordinateSystem {
    fn set_screen_size(&mut self, screen_size: Vector2<u32>) {
        self.screen_size = screen_size;
    }

    fn set_sim_size(&mut self, _sim_size: Vector2<Scalar>) {
        // self.sim_size = sim_size;
    }

    fn sim2screen(&self, position: Position) -> Vector2f {
        Vector2f::new(
            position.x as f32,
            self.screen_size.y as f32 - position.y as f32,
        )
    }

    fn screen2sim(&self, position: Vector2f) -> Position {
        Vector2::new(
            position.x as Scalar,
            self.screen_size.y as Scalar - position.y as Scalar,
        )
    }
}
