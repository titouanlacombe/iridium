use std::time::{Duration, Instant};

use log::info;
use psutil::process::Process;
use sfml::window::Event;

use crate::{
    renderer::Renderer,
    simulation::{Simulation, SimulationRunner},
    timer::Timer,
};

type EventHandler = Box<dyn FnMut(&mut Box<dyn Renderer>, &mut Simulation, &mut bool, &Event)>;

pub struct IridiumMain {
    sim: Simulation,
    renderer: Box<dyn Renderer>,
    sim_runner: Box<dyn SimulationRunner>,
    event_handler: EventHandler,

    steps_per_frame: usize,
    running: bool,

    log_interval: Duration,
    log_separator: String,

    process: Process,
    num_cpus: u64,
}

impl IridiumMain {
    pub fn new(
        sim: Simulation,
        renderer: Box<dyn Renderer>,
        sim_runner: Box<dyn SimulationRunner>,
        event_handler: EventHandler,
        steps_per_frame: usize,
        log_interval: Duration,
    ) -> Self {
        Self {
            sim,
            renderer,
            sim_runner,
            event_handler,
            steps_per_frame,
            running: true,
            log_interval,
            log_separator: "-".repeat(80),
            process: Process::new(std::process::id()).unwrap(),
            num_cpus: psutil::cpu::cpu_count(),
        }
    }

    // Default loop for quick prototyping
    pub fn run(&mut self) {
        let mut last_log = Instant::now();

        let mut sim_elapsed = Duration::ZERO;
        let mut render_elapsed = Duration::ZERO;
        let mut events_elapsed = Duration::ZERO;
        let mut frame_count = 0;

        while self.running {
            let mut timer = Timer::new_now();
            frame_count += 1;

            for _ in 0..self.steps_per_frame {
                self.sim_runner.step(&mut self.sim);
            }
            sim_elapsed += timer.lap();

            let events = self.renderer.events();
            for event in events {
                (self.event_handler)(&mut self.renderer, &mut self.sim, &mut self.running, &event);
            }
            events_elapsed += timer.lap();

            self.renderer.render(&self.sim.particles);
            render_elapsed += timer.lap();

            let log_elapsed = last_log.elapsed();
            if log_elapsed >= self.log_interval {
                self.report(
                    frame_count,
                    log_elapsed,
                    sim_elapsed,
                    render_elapsed,
                    events_elapsed,
                );

                // Reset counters
                last_log = Instant::now();
                sim_elapsed = Duration::ZERO;
                render_elapsed = Duration::ZERO;
                events_elapsed = Duration::ZERO;
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
        events_elapsed: Duration,
    ) {
        // Separator
        info!("{}", self.log_separator);

        // Frames
        let log_elapsed_sec = log_elapsed.as_secs_f64();
        info!(
            "{} frames in {:.3} s (~{:.1} fps)",
            frame_count,
            log_elapsed_sec,
            frame_count as f64 / log_elapsed_sec
        );

        // Timings
        let sim_elapsed_sec = sim_elapsed.as_secs_f64();
        let sim_steps = self.steps_per_frame * frame_count;
        let render_elapsed_sec = render_elapsed.as_secs_f64();
        let events_elapsed_sec = events_elapsed.as_secs_f64();
        info!(
            "{:.2} ms/frame ({:.2} ms/sim step (x{} it), {:.2} ms/render, {:.2} ms/window events)",
            log_elapsed_sec * 1000. / frame_count as f64,
            sim_elapsed_sec * 1000. / sim_steps as f64,
            self.steps_per_frame,
            render_elapsed_sec * 1000. / frame_count as f64,
            events_elapsed_sec * 1000. / frame_count as f64
        );

        // Particles
        let particle_count = self.sim.particles.len();
        let system_count = self.sim.systems.len();
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
