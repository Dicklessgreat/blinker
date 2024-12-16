//! no_std led blinking library for embedded systems.
//!
//! provides a [`Blinker`] struct that can control an output pin to create blinking patterns,
//! and supports both finite and infinite blinking [`Schedule`]s with configurable durations.
//!
//! # Features
//! - async/await support
//! - Configurable blink patterns through [`Schedule`]
//! - Support for both finite and infinite blinking sequences
//! - No heap allocation (uses [heapless::Vec](https://docs.rs/heapless/latest/heapless/struct.Vec.html))
//!
//! The main purpose of this library is to provide a simple and efficient way to control an led to create blinking patterns,
//! but it can also be used for any purpose that requires toggling an output pin according to specific patterns.
//!
//! # Example
//! ## blinks with 500ms interval
//! ```ignore
//! async fn blink_task(led_pin: impl StatefulOutputPin) {
//!     let mut blinker = Blinker::<_, 1>::new(led_pin);
//!     // Blink with 500ms interval
//!     let _ = blinker.push_schedule(Schedule::Infinite(Duration::from_millis(500)));
//!     // Run the blink pattern
//!     loop {
//!         let _ = blinker.step().await;
//!     }
//! }
//! ```
//! ## blinks faster when a button is pushed
//! ```ignore
//! async fn blink_task(led_pin: impl StatefulOutputPin, rx: Receiver<Event>) {
//!     let mut blinker = Blinker::<_, 2>::new(led_pin);
//!     // Blink with 500ms interval
//!     let _ = blinker.push_schedule(Schedule::Ininite(Duration::from_millis(500)));
//!     // Run the blink pattern
//!     loop {
//!         if let Either::Second(Event::ButtonPushed) = select(blinker.step().await, rx.recv()).await {
//!             // ignore overflow
//!             let _ = blinker.push_schedule(Schedule::Ininite(Duration::from_millis(100)));
//!         }
//!     }
//! }
//! ```
#![cfg_attr(not(test), no_std)]

use embassy_time::{Duration, Timer};
use embedded_hal::digital::StatefulOutputPin;
use heapless::Vec;

/// controls an output pin to create blinking patterns.
pub struct Blinker<P: StatefulOutputPin, const N: usize> {
    pin: P,
    schedule: Vec<Schedule, N>,
}

impl<P: StatefulOutputPin, const N: usize> Blinker<P, N> {
    /// Create a new `Blinker` struct
    pub fn new(pin: P) -> Self {
        Self {
            pin,
            schedule: Vec::new(),
        }
    }
    /// Push a new schedule to the stack
    /// Returns an error if the stack is full
    pub fn push_schedule(&mut self, schedule: Schedule) -> Result<(), Schedule> {
        self.schedule.push(schedule)
    }
    /// Clears schedules and sets the pin to low.
    /// Returns an error if the pin is in a bad state(check if your environment supports "infallible" GPIO operations)
    pub fn reset(&mut self) -> Result<(), P::Error> {
        self.pin.set_low()?;
        self.schedule.clear();
        Ok(())
    }
    /// Executes one step of the schedule that is on the top of the stack.
    /// If there is no schedule, does nothing(so be careful if you call this function in a loop).
    /// Returns an error if the pin is in a bad state(check if your environment supports "infallible" GPIO operations).
    pub async fn step(&mut self) -> Result<(), P::Error> {
        if let Some(schedule) = self.schedule.last() {
            match schedule {
                Schedule::Finite(_, dur) | Schedule::Infinite(dur) => {
                    self.pin.toggle()?;
                    Timer::after(*dur).await;
                }
            }
        }
        self.decrease_count();
        Ok(())
    }

    fn decrease_count(&mut self) {
        let mut should_pop = false;
        if let Some(Schedule::Finite(ref mut count, _)) = self.schedule.last_mut() {
            if let Some(c) = count.checked_sub(1) {
                *count = c;
            } else {
                should_pop = true;
            }
        }
        if should_pop {
            self.schedule.pop();
        }
    }
}

/// A blinking schedule that can be pushed to the `Blinker`.
/// This represents how you want to blink the pin.
/// see `Blinker::push_schedule`.
pub enum Schedule {
    /// Periodically toggle the pin.
    /// The duration is the time between toggles.
    Infinite(Duration),
    /// Periodically toggle the pin a specified number of times.
    Finite(u32, Duration),
}

#[cfg(test)]
mod tests {
    use super::*;
    use embassy_futures::block_on;
    use embedded_hal_mock::eh1::digital::{Mock as PinMock, State, Transaction};

    #[test]
    fn test_blinker_finite_schedule() {
        let expectations = [
            Transaction::toggle(),
            Transaction::toggle(),
            Transaction::toggle(),
        ];
        let mut pin = PinMock::new(&expectations);
        let mut blinker = Blinker::<_, 2>::new(&mut pin);

        // 3回点滅するスケジュールを追加
        let _ = blinker.push_schedule(Schedule::Finite(2, Duration::from_millis(100)));

        // 3回ステップを実行
        block_on(async {
            blinker.step().await.expect("infallible");
            blinker.step().await.expect("infallible");
            blinker.step().await.expect("infallible");
        });

        // スケジュールが空になっているはず
        assert!(blinker.schedule.is_empty());
        drop(blinker);
        pin.done();
    }

    #[test]
    fn test_blinker_infinite_schedule() {
        let expectations = [
            Transaction::toggle(),
            Transaction::toggle(),
            Transaction::toggle(),
        ];
        let mut pin = PinMock::new(&expectations);
        let mut blinker = Blinker::<_, 2>::new(&mut pin);

        // 無限スケジュールを追加
        let _ = blinker.push_schedule(Schedule::Infinite(Duration::from_millis(100)));

        block_on(async {
            // 3回ステップを実行
            blinker.step().await.expect("infallible");
            blinker.step().await.expect("infallible");
            blinker.step().await.expect("infallible");
        });
        // スケジュールはまだ残っているはず
        assert!(!blinker.schedule.is_empty());
        drop(blinker);
        pin.done();
    }

    #[test]
    fn test_blinker_reset() {
        let expectations = [Transaction::set(State::Low)];
        let mut pin = PinMock::new(&expectations);
        let mut blinker = Blinker::<_, 2>::new(&mut pin);

        let _ = blinker.push_schedule(Schedule::Infinite(Duration::from_millis(100)));

        blinker.reset().expect("infallible");
        assert!(blinker.schedule.is_empty());
        drop(blinker);
        pin.done();
    }
}
