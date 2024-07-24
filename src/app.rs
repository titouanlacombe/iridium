use log::info;
use psutil::process::Process;
use std::time::{Duration, Instant};
use tracy_client::frame_mark;

use crate::{
    rendering::renderer::Renderer,
    simulation::simulation::{Simulation, SimulationRunner},
    utils::timer::Timer,
};

// Regroup all app data in one struct to be edited by the various systems
pub struct AppData {
    pub sim: Simulation,
    pub sim_runner: Box<dyn SimulationRunner>,
    pub steps_per_frame: usize,
    pub log_interval: Duration,
    pub log_separator: String,

    pub running: bool,
    pub stop: bool,
}

pub struct AppMain {
    data: AppData,
    renderer: Box<dyn Renderer>,

    process: Process,
    num_cpus: u64,
}

impl AppMain {
    pub fn new(
        sim: Simulation,
        renderer: Box<dyn Renderer>,
        sim_runner: Box<dyn SimulationRunner>,
        steps_per_frame: usize,
        log_interval: Duration,
    ) -> Self {
        let data = AppData {
            sim,
            sim_runner,
            running: true,
            stop: false,
            steps_per_frame,
            log_interval,
            log_separator: "-".repeat(80),
        };

        Self {
            data,
            renderer,
            process: Process::new(std::process::id()).unwrap(),
            num_cpus: psutil::cpu::cpu_count(),
        }
    }

    // Default loop for quick prototyping
    pub fn run(&mut self) {
        let mut last_log = Instant::now();
        let mut prof_timer = Timer::new_now();

        let mut sim_elapsed = Duration::ZERO;
        let mut render_elapsed = Duration::ZERO;
        let mut frame_count = 0;

        while !self.data.stop {
            let _span = tracy_client::span!("Frame");

            frame_count += 1;
            prof_timer.lap();

            if self.data.running {
                for _ in 0..self.data.steps_per_frame {
                    let _span = tracy_client::span!("Simulation step");

                    self.data.sim_runner.step(&mut self.data.sim);
                }
            }
            sim_elapsed += prof_timer.lap();

            self.renderer.render(&mut self.data);
            render_elapsed += prof_timer.lap();

            frame_mark();

            let log_elapsed = last_log.elapsed();
            if log_elapsed >= self.data.log_interval {
                self.report(frame_count, log_elapsed, sim_elapsed, render_elapsed);

                // Reset counters
                last_log = Instant::now();
                sim_elapsed = Duration::ZERO;
                render_elapsed = Duration::ZERO;
                frame_count = 0;
            }
        }
    }

    fn report(
        &mut self,
        frame_count: usize,
        log_elapsed: Duration,
        sim_elapsed: Duration,
        render_elapsed: Duration,
    ) {
        // Separator
        info!("{}", self.data.log_separator);

        // Frames
        let log_elapsed_sec = log_elapsed.as_secs_f64();
        info!(
            "Rendered {} frames in {:.3} s (~{:.1} fps)",
            frame_count,
            log_elapsed_sec,
            frame_count as f64 / log_elapsed_sec
        );

        // Timings
        let sim_elapsed_sec = sim_elapsed.as_secs_f64();
        let sim_steps = self.data.steps_per_frame * frame_count;
        let render_elapsed_sec = render_elapsed.as_secs_f64();
        info!(
            "Frame: {:.2} ms (Sim: {:.2} ms ({} steps/frame), Render: {:.2} ms)",
            log_elapsed_sec * 1000. / frame_count as f64,
            sim_elapsed_sec * 1000. / sim_steps as f64,
            self.data.steps_per_frame,
            render_elapsed_sec * 1000. / frame_count as f64,
        );

        // Particles
        let particle_count = self.data.sim.particles.len();
        let system_count = self.data.sim.systems.len();
        info!(
            "{:.2e} particles ({:.2} ns/particle), {} systems",
            particle_count,
            sim_elapsed_sec * 1E9 / (sim_steps as f64 * particle_count as f64),
            system_count
        );

        // Process info
        let cpu_percent = self.process.cpu_percent().unwrap();
        let memory = self.process.memory_info().unwrap();
        info!(
            "CPU: {:.1}% ({} CPUs), Memory: {:.1} MB",
            cpu_percent / self.num_cpus as f32,
            self.num_cpus,
            memory.rss() as f64 / 1e6
        );
    }
}

// TODO in the future move to builder pattern (set_max_fps -> optional<u64>, set_min_duration -> optional<Duration>)
pub fn max_fps(fps: u64) -> Option<Duration> {
    Some(Duration::from_micros(1_000_000 / fps))
}
