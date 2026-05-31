//! Profiler implementation

use std::time::{Duration, Instant};

/// Profile sample
#[derive(Debug, Clone)]
pub struct ProfileSample {
    pub name: String,
    pub duration: Duration,
    pub timestamp: Instant,
}

/// Profile report
#[derive(Debug, Clone)]
pub struct ProfileReport {
    pub samples: Vec<ProfileSample>,
    pub total_duration: Duration,
}

impl ProfileReport {
    pub fn average_duration(&self) -> Duration {
        if self.samples.is_empty() {
            return Duration::ZERO;
        }
        self.total_duration / self.samples.len() as u32
    }
}

/// Profiler
pub struct Profiler {
    samples: Vec<ProfileSample>,
    start_time: Option<Instant>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn record(&mut self, name: String, duration: Duration) {
        self.samples.push(ProfileSample {
            name,
            duration,
            timestamp: Instant::now(),
        });
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    pub fn report(&self) -> ProfileReport {
        let total_duration = self.samples.iter().map(|s| s.duration).sum();

        ProfileReport {
            samples: self.samples.clone(),
            total_duration,
        }
    }

    pub fn clear(&mut self) {
        self.samples.clear();
        self.start_time = None;
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}
