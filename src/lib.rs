#![cfg_attr(not(test), no_std)]

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
    pub fn reset(&mut self) -> Result<(), P::Error> {
        self.pin.set_low()?;
        self.schedule.clear();
        Ok(())
    }
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

pub enum Schedule {
    Infinite(Duration),
    Finite(u32, Duration),
    // Sequence(Vec<, 2>),
    // Random(Vec<u32>),
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
        let pin = PinMock::new(&expectations);
        let mut blinker = Blinker::<_, 2>::new(pin);

        let _ = blinker.push_schedule(Schedule::Infinite(Duration::from_millis(100)));

        blinker.reset().expect("infallible");
        assert!(blinker.schedule.is_empty());
    }
}
