use rayon::prelude::*;

pub trait Integrator<T: Clone + Send + Sync> {
    fn integrate_vec(&self, values: &Vec<T>, result: &mut Vec<T>, dt: f64);
}

pub struct GaussianIntegrator;

impl<T: Clone + Send + Sync + std::ops::AddAssign + std::ops::Mul<f64, Output = T>> Integrator<T>
    for GaussianIntegrator
{
    fn integrate_vec(&self, values: &Vec<T>, result: &mut Vec<T>, dt: f64) {
        values
            .par_iter()
            .zip(result.par_iter_mut())
            .for_each(|(value, result)| {
                *result += (*value).clone() * dt;
            });
    }
}
