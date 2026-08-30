use rayon::prelude::*;

use super::types::Scalar;

pub trait Integrator<T: Clone + Send + Sync> {
    fn integrate_vec(&self, values: &[T], result: &mut [T], dt: Scalar);
}

pub struct GaussianIntegrator;

impl<T: Clone + Send + Sync + std::ops::AddAssign + std::ops::Mul<Scalar, Output = T>> Integrator<T>
    for GaussianIntegrator
{
    fn integrate_vec(&self, values: &[T], result: &mut [T], dt: Scalar) {
        values
            .par_iter()
            .zip(result.par_iter_mut())
            .for_each(|(value, result)| {
                *result += (*value).clone() * dt;
            });
    }
}
