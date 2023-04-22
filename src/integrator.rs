use std::ops::{AddAssign, Mul};

use crate::types::Time;

pub trait Integrator<T> {
    fn integrate_vec(&self, values: &Vec<T>, result: &mut Vec<T>, dt: Time);
}

pub struct GaussianIntegrator;

impl GaussianIntegrator {
    pub fn new() -> Self {
        Self
    }
}

impl<T: AddAssign<T> + Mul<Time, Output = T> + Copy> Integrator<T> for GaussianIntegrator {
    fn integrate_vec(&self, values: &Vec<T>, result: &mut Vec<T>, dt: Time) {
        for (value, result) in values.iter().zip(result.iter_mut()) {
            *result += *value * dt;
        }
    }
}
