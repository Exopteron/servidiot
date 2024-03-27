use std::{num::NonZeroU64, time::{Duration, Instant}};

pub struct TickLoop<F: FnMut() -> bool> {
    per_second: NonZeroU64,
    func: F
}

impl<F: FnMut() -> bool> TickLoop<F> {
    pub fn new(per_second: NonZeroU64, func: F) -> Self {
        Self {
            per_second,
            func
        }
    }
    pub fn run(mut self) {
        let duration = Duration::from_millis(1000 / self.per_second.get());

        loop {
            let start = Instant::now();
            if !(self.func)() {
                break;
            }
            let took = start.elapsed();
            if took < duration {
                std::thread::sleep(duration - took);
            } else {
                tracing::error!("Tick took too long! {}ms", took.as_millis());
            }
        }
    }
}