use super::nano::Time;
use spin_sleep::{SpinSleeper, SpinStrategy};
use std::time::*;
use uom::si::time::nanosecond;

pub fn now() -> Time {
    Time::new::<nanosecond>(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64,
    )
}

pub fn next_millisecond(t: Time) -> Time {
    // Goal round `t` up to the next millisecond (1_000_000) nanoseconds and get that time.
    let round_down_amount = t % Time::new::<nanosecond>(1_000_000);
    let round_up_amount = Time::new::<nanosecond>(1_000_000) - round_down_amount;
    round_up_amount
}

// This exists for short sleeping durations, like for example the distributed sleep (~254 ns)
pub async fn accurate_sleep(duration: std::time::Duration) {
    tokio::task::spawn_blocking(move || {
        SpinSleeper::new(100_000)
            .with_spin_strategy(SpinStrategy::YieldThread)
            .sleep(duration);
    })
    .await
    .expect("Sleeping failed!");
}
