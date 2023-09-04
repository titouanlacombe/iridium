use iridium::examples::benchmark1;

// Multi-threading experiment
use rand::Rng;
use rayon::prelude::*;
fn _mt_exp() {
    // Heavy computation
    fn heavy_compute(value: &mut u128) {
        for _ in 0..1 {
            *value += *value * *value;
        }
    }

    let mut array = vec![0; 1_000_000];
    let mut rng = rand::thread_rng();
    for value in array.iter_mut() {
        *value = rng.gen();
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
        "Single thread: {} us\nMulti thread: {} us",
        single_thread_time, multi_thread_time
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
