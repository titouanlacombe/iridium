use nalgebra::Vector2;

pub type Scalar = f64;

pub type Mass = Scalar;
pub type Time = Scalar;
pub type Energy = Scalar;
pub type Temperature = Scalar;
pub type Length = Scalar;

pub type Position = Vector2<Scalar>;
pub type Velocity = Vector2<Scalar>;
pub type Acceleration = Vector2<Scalar>;
pub type Force = Vector2<Scalar>;

// TODO struct Color & impl for easy manipulation (use lib maybe)
pub type Color = (Scalar, Scalar, Scalar, Scalar);
