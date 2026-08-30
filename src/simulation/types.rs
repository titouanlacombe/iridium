use nalgebra::Vector2;

#[cfg(feature = "f32")]
pub type Scalar = f32;
#[cfg(not(feature = "f32"))]
pub type Scalar = f64;

pub const PI: Scalar = std::f64::consts::PI as Scalar;

pub type Mass = Scalar;
pub type Time = Scalar;
pub type Energy = Scalar;
pub type Temperature = Scalar;
pub type Length = Scalar;

pub type Position = Vector2<Scalar>;
pub type Velocity = Vector2<Scalar>;
pub type Acceleration = Vector2<Scalar>;
pub type Force = Vector2<Scalar>;
