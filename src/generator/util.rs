use crate::generator::nano::Nanosecond;
use spin_sleep::{SpinSleeper, SpinStrategy};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> Nanosecond {
    // Fine, since it's not my fucking problem if someone uses this in 270 years.
    #[allow(clippy::cast_possible_truncation)]
    Nanosecond(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64,
    )
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
