#![no_std]

use embassy_time::Duration;
use embedded_hal::digital::StatefulOutputPin;
use heapless::Vec;

pub struct Blinker<P: StatefulOutputPin, const N: usize> {
    pin: P,
    schedule: Vec<Schedule, N>,
}

pub struct Schedule {
    pub interval: Form,
    pub dur: Duration,
}

pub enum Form {
    Infinite,
    Finite(u32),
    // Sequence(Vec<, 2>),
    // Random(Vec<u32>),
}
