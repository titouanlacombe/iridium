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

// Multithreading experiment
// impl Integrator<Vector2<Scalar>> for GaussianIntegrator {
//     fn integrate_vec(
//         &self,
//         values: &Vec<Vector2<Scalar>>,
//         result: &mut Vec<Vector2<Scalar>>,
//         dt: Time,
//     ) {
//         let iterator = values.par_iter().zip(result.par_iter_mut());
//         // println!("Number of threads: {}", rayon::current_num_threads());
//         let timer = Instant::now();
//         // iterator.for_each(|(value, result)| {
//         //     *result += *value * dt;
//         // });
//         for (value, result) in values.iter().zip(result.iter_mut()) {
//             *result += *value * dt;
//         }
//         println!("Time: {} µs", timer.elapsed().as_micros());
//     }
// }
