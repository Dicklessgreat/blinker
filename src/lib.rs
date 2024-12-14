#![no_std]

use embedded_hal::digital::OutputPin;
use heapless::Vec;

pub struct Blinker<P: OutputPin, const N: usize> {
    pin: P,
    schedule: Vec<Schedule, N>
}

pub struct Schedule {
    interval: Interval,
}

pub enum Interval {
    Infinite,
    Finite(u32),
    Sequence(Vec<u32, 2>),
    // Random(Vec<u32>),
}