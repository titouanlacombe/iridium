use rayon::prelude::*;

use super::types::Scalar;

// Per-element scaled integration: result += values * scale * dt in one pass.
// `None` scale takes the unscaled branch (result += values * dt) with no
// per-element multiply and no scale buffer - the scaled variant would otherwise
// pay an unfoldable `* 1.0` and an extra memory read on every element.
pub trait Integrator<T: Clone + Send + Sync> {
    fn integrate_vec(&self, values: &[T], scale: Option<&[Scalar]>, result: &mut [T], dt: Scalar)
    where
        T: std::ops::AddAssign + std::ops::Mul<Scalar, Output = T>;
}

pub struct GaussianIntegrator;

impl<T: Clone + Send + Sync + std::ops::AddAssign + std::ops::Mul<Scalar, Output = T>> Integrator<T>
    for GaussianIntegrator
{
    fn integrate_vec(&self, values: &[T], scale: Option<&[Scalar]>, result: &mut [T], dt: Scalar) {
        // The match selects the per-element compute (scaled vs plain); each arm
        // calls the shared generic loop with its concrete closure, so both
        // instantiations are monomorphized and inlined (no dynamic dispatch).
        match scale {
            Some(scale) => Self::integrate_loop(values, result, |i, value, result| {
                *result += (*value).clone() * scale[i] * dt;
            }),
            None => Self::integrate_loop(values, result, |_i, value, result| {
                *result += (*value).clone() * dt;
            }),
        }
    }
}

impl GaussianIntegrator {
    fn integrate_loop<T, F>(values: &[T], result: &mut [T], compute: F)
    where
        T: Clone + Send + Sync + std::ops::AddAssign + std::ops::Mul<Scalar, Output = T>,
        F: Fn(usize, &T, &mut T) + Sync,
    {
        values
            .par_iter()
            .enumerate()
            .zip(result.par_iter_mut())
            .for_each(|((i, value), result)| compute(i, value, result));
    }
}
