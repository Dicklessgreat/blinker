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
        Ok(())
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
