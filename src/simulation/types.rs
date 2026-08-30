use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use nalgebra::Vector2;
use wide::{f32x8, f64x4};

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

// Minimal SIMD abstraction over wide's f64x4 / f32x8 (one AVX2 register per lane count).
// All scalar arrays are [Scalar; 8] scratch buffers: the vector types pack/unpack
// their first LANES elements.
pub trait SimdVec:
    Copy + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self> + AddAssign<Self> + Neg<Output = Self>
{
    type Scalar;
    const LANES: usize;

    fn splat(value: Self::Scalar) -> Self;
    fn from_8(values: &[Self::Scalar; 8]) -> Self;
    fn write_8(self, out: &mut [Self::Scalar; 8]);
    fn sqrt(self) -> Self;
    fn powv(self, exponent: Self) -> Self;
    fn mask_lt(self, rhs: Self) -> Self;
    fn mask_le(self, rhs: Self) -> Self;
    fn mask_gt(self, rhs: Self) -> Self;
    fn mask_ge(self, rhs: Self) -> Self;
    fn select(self, if_true: Self, if_false: Self) -> Self;
    fn to_bitmask(self) -> u32;
}

impl SimdVec for f64x4 {
    type Scalar = f64;
    const LANES: usize = 4;

    fn splat(value: f64) -> Self {
        f64x4::splat(value)
    }
    fn from_8(values: &[f64; 8]) -> Self {
        f64x4::new([values[0], values[1], values[2], values[3]])
    }
    fn write_8(self, out: &mut [f64; 8]) {
        let a = self.to_array();
        out[0] = a[0];
        out[1] = a[1];
        out[2] = a[2];
        out[3] = a[3];
    }
    fn sqrt(self) -> Self {
        self.sqrt()
    }
    fn powv(self, exponent: Self) -> Self {
        self.powf_simd(exponent)
    }
    fn mask_lt(self, rhs: Self) -> Self {
        self.simd_lt(rhs)
    }
    fn mask_le(self, rhs: Self) -> Self {
        self.simd_le(rhs)
    }
    fn mask_gt(self, rhs: Self) -> Self {
        self.simd_gt(rhs)
    }
    fn mask_ge(self, rhs: Self) -> Self {
        self.simd_ge(rhs)
    }
    fn select(self, if_true: Self, if_false: Self) -> Self {
        self.select(if_true, if_false)
    }
    fn to_bitmask(self) -> u32 {
        self.to_bitmask()
    }
}

impl SimdVec for f32x8 {
    type Scalar = f32;
    const LANES: usize = 8;

    fn splat(value: f32) -> Self {
        f32x8::splat(value)
    }
    fn from_8(values: &[f32; 8]) -> Self {
        f32x8::new(*values)
    }
    fn write_8(self, out: &mut [f32; 8]) {
        *out = self.to_array();
    }
    fn sqrt(self) -> Self {
        self.sqrt()
    }
    fn powv(self, exponent: Self) -> Self {
        self.powf_simd(exponent)
    }
    fn mask_lt(self, rhs: Self) -> Self {
        self.simd_lt(rhs)
    }
    fn mask_le(self, rhs: Self) -> Self {
        self.simd_le(rhs)
    }
    fn mask_gt(self, rhs: Self) -> Self {
        self.simd_gt(rhs)
    }
    fn mask_ge(self, rhs: Self) -> Self {
        self.simd_ge(rhs)
    }
    fn select(self, if_true: Self, if_false: Self) -> Self {
        self.select(if_true, if_false)
    }
    fn to_bitmask(self) -> u32 {
        self.to_bitmask()
    }
}

#[cfg(feature = "f32")]
pub type Simd = f32x8;
#[cfg(not(feature = "f32"))]
pub type Simd = f64x4;

// Convert a SIMD mask (all-ones / all-zeros lanes) into 1.0 / 0.0 lanes
pub fn mask_to_01<V: SimdVec<Scalar = Scalar>>(mask: V) -> V {
    mask.select(V::splat(1.0), V::splat(0.0))
}

// Zero out masked lanes by selection, not multiplication: masked lanes may hold
// NaN (e.g. the batch's own particle at distance 0), and NaN * 0.0 is NaN.
// `mask` uses the 1.0 / 0.0 convention; convert to an all-ones/zeros mask first
// because select() tests for the all-ones bit pattern.
pub fn masked<V: SimdVec<Scalar = Scalar>>(mask: V, value: V) -> V {
    mask.mask_gt(V::splat(0.0))
        .select(value, V::splat(0.0))
}

// r^(-power) with fast paths for common powers (exact integer powers via muls)
pub fn repulsion_inv_pow<V: SimdVec<Scalar = Scalar>>(power: i32, r: V) -> V {
    match power {
        1 => V::splat(1.0) / r,
        2 => {
            let r2 = r * r;
            V::splat(1.0) / r2
        }
        3 => {
            let r2 = r * r;
            V::splat(1.0) / (r2 * r)
        }
        4 => {
            let r2 = r * r;
            V::splat(1.0) / (r2 * r2)
        }
        6 => {
            let r2 = r * r;
            V::splat(1.0) / (r2 * r2 * r2)
        }
        p => r.powv(V::splat(-(p as Scalar))),
    }
}
