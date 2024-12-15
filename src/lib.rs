#![no_std]

use embassy_time::{Duration, Timer};
use embedded_hal::digital::StatefulOutputPin;
use heapless::Vec;

pub struct Blinker<P: StatefulOutputPin, const N: usize> {
    pin: P,
    schedule: Vec<Schedule, N>,
}

impl<P: StatefulOutputPin, const N: usize> Blinker<P, N> {
    pub fn new(pin: P) -> Self {
        Self {
            pin,
            schedule: Vec::new(),
        }
    }
    pub fn push_schedule(&mut self, schedule: Schedule) -> Result<(), Schedule> {
        self.schedule.push(schedule)
    }
    pub async fn run(&mut self) -> Result<(), P::Error> {
        if let Some(schedule) = self.schedule.first() {
            match schedule.interval {
                Form::Finite(_, dur) | Form::Infinite(dur) => {
                    self.pin.toggle()?;
                    Timer::after(dur).await;
                }
            }
        }
        self.decrease_count();
        Ok(())
    }

    fn decrease_count(&mut self) {
        let mut should_pop = false;
        if let Some(schedule) = self.schedule.first_mut() {
            if let Form::Finite(ref mut count, _) = schedule.interval {
                if let Some(c) = count.checked_sub(1) {
                    *count = c;
                } else {
                    should_pop = true;
                }
            }
        }
        if should_pop {
            self.schedule.pop();
        }
    }
}

pub struct Schedule {
    pub interval: Form,
}

pub enum Form {
    Infinite(Duration),
    Finite(u32, Duration),
    // Sequence(Vec<, 2>),
    // Random(Vec<u32>),
}
