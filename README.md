# Blinker

A no_std led blinking library for embedded systems.

## Features

- Async/await support
- Configurable blink patterns through `Schedule`
- Support for both finite and infinite blinking sequences
- No heap allocation (uses [heapless](https://github.com/rust-embedded/heapless.git) Vec)

```rust
use blinker::{Blinker, Schedule};
use embassy_time::Duration;
use embedded_hal::digital::StatefulOutputPin;

async fn blink_task(led_pin: impl StatefulOutputPin) {
    let mut blinker = Blinker::<_, 1>::new(led_pin);
    // Blink with 500ms interval
    let _ = blinker.push_schedule(Schedule::Infinite(Duration::from_millis(500)));
    // Run the blink pattern
    loop {
        let _ = blinker.step().await;
    }
}
```

See [docs](https://docs.rs/blinker/latest/blinker/) for more details.
