use chrono::prelude::*;
use spin_sleep::{SpinSleeper, SpinStrategy};

pub fn now() -> chrono::DateTime<Utc> {
    Utc::now()
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
