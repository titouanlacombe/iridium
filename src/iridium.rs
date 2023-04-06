use log::info;
use sfml::window::Event;
use std::time::{Duration, Instant};

use crate::{
    renderer::BasicRenderer,
    simulation::{Simulation, SimulationRunner},
    timer::Timer,
};

type EventHandler = Box<dyn FnMut(&mut BasicRenderer, &mut Simulation, &mut bool, &Event)>;

pub struct IridiumMain {
    pub renderer: BasicRenderer,
    pub sim: Simulation,
    pub sim_runner: Box<dyn SimulationRunner>,
    pub event_handler: EventHandler,

    pub log_interval: Duration,
    pub steps_per_frame: usize,
    pub running: bool,
}

impl IridiumMain {
    pub fn new(
        renderer: BasicRenderer,
        sim: Simulation,
        sim_runner: Box<dyn SimulationRunner>,
        event_handler: EventHandler,
        log_interval: Duration,
        steps_per_frame: usize,
    ) -> Self {
        Self {
            renderer,
            sim,
            sim_runner,
            event_handler,
            log_interval,
            steps_per_frame,
            running: true,
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

            self.renderer.render(&self.sim.particles);
            render_elapsed += timer.lap();

            let events = self.renderer.events();
            for event in events {
                (self.event_handler)(&mut self.renderer, &mut self.sim, &mut self.running, &event);
            }
            events_elapsed += timer.lap();

            let log_elapsed = last_log.elapsed();
            if log_elapsed >= self.log_interval {
                let log_elapsed_sec = log_elapsed.as_secs_f64();
                let frame_time_av = log_elapsed_sec / frame_count as f64;
                let particle_count = self.sim.particles.len();

                info!(
                    "\n{} steps in {:.2} s (~{:.2} fps)\n\
					{:.2} ms/step ({:.2} ms/sim, {:.2} ms/render, {:.2} ms/events)\n\
					{:.2e} particles ({:.2} µs/particle)\n\
					{} systems",
                    frame_count,
                    log_elapsed_sec,
                    1. / frame_time_av,
                    frame_time_av * 1000.,
                    sim_elapsed.as_secs_f64() * 1000. / frame_count as f64,
                    render_elapsed.as_secs_f64() * 1000. / frame_count as f64,
                    events_elapsed.as_secs_f64() * 1000. / frame_count as f64,
                    particle_count,
                    ((log_elapsed_sec * 1e6) / frame_count as f64) / particle_count as f64,
                    self.sim.systems.len()
                );

                last_log = Instant::now();
                sim_elapsed = Duration::ZERO;
                render_elapsed = Duration::ZERO;
                events_elapsed = Duration::ZERO;
                frame_count = 0;
            }
        }
    }
}

// TODO move to facade
pub fn max_fps(fps: u64) -> Option<Duration> {
    Some(Duration::from_micros(1_000_000 / fps))
}
