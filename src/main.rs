use iridium::examples::benchmark1;

// Multi-threading experiment
use rand::Rng;
use rayon::prelude::*;
fn _mt_exp() {
    // Heavy computation for f64
    fn heavy_compute(value: &mut f64) {
        for _ in 0..1 {
            *value += *value * *value;
        }
    }

    let mut array = vec![0.; 1_000_000];
    let mut rng = rand::thread_rng();
    for i in 0..array.len() {
        array[i] = rng.gen_range(0. ..1.);
    }

    let pool = rayon::ThreadPoolBuilder::new().build().unwrap();

    let single_thread_time = std::time::Instant::now();
    array.iter_mut().for_each(|value| {
        heavy_compute(value);
    });
    let single_thread_time = single_thread_time.elapsed().as_micros();

    let multi_thread_time = std::time::Instant::now();
    pool.install(|| {
        array.par_chunks_mut(10_000).for_each(|chunk| {
            chunk.iter_mut().for_each(|value| {
                heavy_compute(value);
            });
        });
    });
    let multi_thread_time = multi_thread_time.elapsed().as_micros();

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

    // _mt_exp();
}
