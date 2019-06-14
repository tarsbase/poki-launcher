use derive_new::*;
use log::*;
use std::cmp::Ordering;

use std::fmt::Display;
use std::process;
use std::time::SystemTime;
use super::App;

#[derive(new)]
struct Items {
    items: Vec<App>,
    reference_time: f64,
    half_life: f32,
}

#[allow(dead_code)]
impl Items {
    fn sort(&mut self) {
        self.items.sort_unstable_by(|left, right| {
            left.score
                .partial_cmp(&right.score)
                .unwrap_or(Ordering::Less)
        });
    }

    fn secs_elapsed(&self) -> f32 {
        (current_time_secs() - self.reference_time) as f32
    }

    fn update_score(&mut self, idx: usize, weight: f32) {
        let elapsed = self.secs_elapsed();
        self.items
            .get_mut(idx)
            .unwrap()
            .update_frecency(weight, elapsed, self.half_life);
    }
}


#[allow(dead_code)]
impl App {
    fn get_frecency(&self, elapsed: f32, half_life: f32) -> f32 {
        self.score / 2.0f32.powf(elapsed / half_life)
    }

    fn set_frecency(&mut self, new: f32, elapsed: f32, half_life: f32) {
        self.score = new * 2.0f32.powf(elapsed / half_life);
    }

    fn update_frecency(&mut self, weight: f32, elapsed: f32, half_life: f32) {
        self.set_frecency(
            self.get_frecency(elapsed, half_life) + weight,
            elapsed,
            half_life,
        );
    }
}

/// Return the current time in seconds as a float
#[allow(dead_code)]
pub fn current_time_secs() -> f64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => (n.as_secs() as u128 * 1000 + n.subsec_millis() as u128) as f64 / 1000.0,
        Err(e) => {
            error!("invalid system time: {}", e);
            process::exit(1);
        }
    }
}
