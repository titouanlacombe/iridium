use nalgebra::{Matrix3, Rotation2, Scale2, Translation2, Vector2};

use crate::types::Position;

pub trait Camera: Sync {
    fn sim2screen(&self, position: Position) -> Position;
    fn screen2sim(&self, position: Position) -> Position;

    fn screen_size(&mut self) -> &mut Vector2<u32>;
    fn offset(&mut self) -> &mut Vector2<f64>;
    fn zoom(&mut self) -> &mut f64;
    fn rotation(&mut self) -> &mut f64;

    // Always call this function before using the camera
    fn update_matrices(&mut self);
}

pub struct BasicCamera {
    // Store matrices for faster computation
    sim2screen_matrix: Matrix3<f64>,
    screen2sim_matrix: Matrix3<f64>,

    // Camera parameters
    screen_size: Vector2<u32>,
    offset: Vector2<f64>,
    zoom: f64,
    rotation: f64,
}

impl BasicCamera {
    pub fn new(screen_size: Vector2<u32>, offset: Vector2<f64>, zoom: f64, rotation: f64) -> Self {
        let mut obj = Self {
            sim2screen_matrix: Matrix3::zeros(),
            screen2sim_matrix: Matrix3::zeros(),
            screen_size,
            offset,
            zoom,
            rotation,
        };
        obj.update_matrices();
        obj
    }
}

impl Camera for BasicCamera {
    fn sim2screen(&self, position: Position) -> Position {
        (self.sim2screen_matrix * position.push(1.0)).xy()
    }

    fn screen2sim(&self, position: Position) -> Position {
        (self.screen2sim_matrix * position.push(1.0)).xy()
    }

    fn update_matrices(&mut self) {
        let zoom = self.zoom;
        let rotation = self.rotation;
        let screen_center = self.screen_size.map(|x| x as f64 / 2.0);

        // Create the transformation sequence
        self.sim2screen_matrix = Translation2::new(screen_center.x, screen_center.y)
            .to_homogeneous()
            * Scale2::new(zoom, zoom).to_homogeneous()
            * Rotation2::new(rotation).to_homogeneous()
            * Translation2::new(-screen_center.x, -screen_center.y).to_homogeneous()
            * Scale2::new(1.0, -1.0).to_homogeneous()
            * Translation2::new(0.0, -(self.screen_size.y as f64)).to_homogeneous()
            * Translation2::new(self.offset.x, self.offset.y).to_homogeneous();

        self.screen2sim_matrix = self.sim2screen_matrix.try_inverse().unwrap();
    }

    fn screen_size(&mut self) -> &mut Vector2<u32> {
        &mut self.screen_size
    }

    fn offset(&mut self) -> &mut Vector2<f64> {
        &mut self.offset
    }

    fn zoom(&mut self) -> &mut f64 {
        &mut self.zoom
    }

    fn rotation(&mut self) -> &mut f64 {
        &mut self.rotation
    }
}
