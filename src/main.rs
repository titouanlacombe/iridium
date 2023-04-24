use iridium::examples::benchmark1;

// Multi-threading experiment
use rayon::prelude::*;
fn _mt_exp() {
    // Heavy computation for f64
    fn heavy_compute(value: &mut f64) {
        for _ in 0..1000 {
            *value += *value * *value;
        }
    }

    // Initialize array with random values
    let mut array = vec![0.; 1_000_000];
    for i in 0..array.len() {
        array[i] = rand::random::<f64>();
    }

    // Single-threaded implementation
    let mut timer = std::time::Instant::now();
    array.iter_mut().for_each(|value| {
        heavy_compute(value);
    });
    let single_thread_time = timer.elapsed().as_micros();

    // Multi-threaded implementation
    timer = std::time::Instant::now();
    array.par_iter_mut().for_each(|value| {
        heavy_compute(value);
    });
    let multi_thread_time = timer.elapsed().as_micros();

    println!(
        "Ratio: {}/{} = {}",
        single_thread_time,
        multi_thread_time,
        single_thread_time as f64 / multi_thread_time as f64
    );
}

fn main() {
    // Configure logging
    env_logger::builder()
        .format_timestamp(None)
        .format_level(true)
        .init();

    // Run simulation with renderer loop
    benchmark1().run();

    // mt_exp();
}
