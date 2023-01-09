use chrono::prelude::*;
use std::mem::replace;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::Mutex;

mod checks;
mod util;

use crate::HexaFreezeError;
use crate::{constants, error::HexaFreezeResult};

#[derive(Clone)]
pub struct Generator {
    epoch: DateTime<Utc>,
    node_id: i64,
    increment: Arc<Mutex<i64>>,

    last_reset: Arc<Mutex<DateTime<Utc>>>,
    distribute_sleep: Arc<AtomicBool>,
}

impl Generator {
    /// Creates a new generator with your desired configuration.
    /// 
    /// # Example
    /// ```rust
    /// use hexafreeze::{Generator, DEFAULT_EPOCH};
    /// 
    /// let generator = Generator::new(0, *DEFAULT_EPOCH);
    /// ```
    /// 
    /// # Errors
    /// * When `node_id` is bigger than 1024
    /// * When the epoch is more than ~69 years ago.
    /// * When the epoch is in the future.
    // Ok since it it a string literal and this function is unit tested to not panic.
    #[allow(clippy::missing_panics_doc)]
    pub fn new(node_id: i64, epoch: DateTime<Utc>) -> HexaFreezeResult<Self> {
        checks::check_node_id(node_id)?;
        checks::check_epoch(epoch)?;

        Ok(Self {
            epoch,
            node_id,
            increment: Arc::new(Mutex::new(0)),
            last_reset: Arc::new(Mutex::new(
                DateTime::parse_from_rfc3339("0000-01-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            )),
            distribute_sleep: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Generates a new snowflake ID
    /// 
    /// # Example
    /// ```rust
    /// use hexafreeze::{Generator, DEFAULT_EPOCH};
    /// 
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// let generator = Generator::new(0, *DEFAULT_EPOCH).unwrap();
    /// 
    /// let id = generator.generate().await.unwrap();
    /// # })
    /// ```
    /// 
    /// # Errors
    /// It is generally ok to call `unwrap()` on this Result.
    /// Since it only errors ...
    /// * When the epoch is more than ~69 years ago.
    /// * When you have generated more than `9_223_372_036_854_775_807` ids. (In total, for this generator)
    /// * When your clock jumps backward in time a significant amount.
    pub async fn generate(&self) -> HexaFreezeResult<i64> {
        let mut i = self.increment.lock().await;
        self.distribute_sleep().await;
        let (seq, now) = self.get_sequence(i.deref_mut()).await?;
        drop(i);

        self.create_id(now, seq)
    }

    async fn distribute_sleep(&self) {
        if self.distribute_sleep.load(Ordering::Relaxed) {
            tokio::task::spawn_blocking(|| {
                spin_sleep::SpinSleeper::new(100_000)
                    .with_spin_strategy(spin_sleep::SpinStrategy::YieldThread)
                    .sleep(constants::DISTRIBUTED_SLEEP_TIME);
            })
            .await
            .unwrap();
        }
    }

    async fn get_sequence(&self, inc: &mut i64) -> HexaFreezeResult<(i64, DateTime<Utc>)> {
        let now = util::now();
        let seq = *inc % constants::RESET_INCREMENT;
        if *inc == i64::MAX {
            return Err(HexaFreezeError::Surpassed64BitLimit);
        }
        *inc += 1;

        if seq == 0 {
            let last = replace(self.last_reset.lock().await.deref_mut(), now);

            let delta = now - last;

            if delta < chrono::Duration::seconds(0) {
                return Err(HexaFreezeError::ClockWentBackInTime);
            }

            if delta < chrono::Duration::milliseconds(1) {
                // Safe to unwrap, because we know its below a millisecond and that it's bigger than 0.
                tokio::time::sleep(delta.to_std().unwrap()).await;
                self.distribute_sleep.store(true, Ordering::Relaxed);

                // No .abs(), because we know its bigger than 0
                return Ok((seq, now + delta));
            }

            self.distribute_sleep.store(false, Ordering::Relaxed);
        }

        Ok((seq, now))
    }

    fn create_id(&self, now: DateTime<Utc>, seq: i64) -> HexaFreezeResult<i64> {
        // We know, that the epoch cant be in the future, since it's checked at when a generator is created.
        let ts = now - self.epoch;

        if ts > chrono::Duration::from_std(constants::MAX_TIMESTAMP).unwrap() {
            return Err(HexaFreezeError::EpochTooFarInThePast);
        }

        Ok((ts.num_milliseconds() << constants::TIMESTAMP_SHIFT)
            | (self.node_id << constants::INSTANCE_SHIFT)
            | (seq << constants::SEQUENCE_SHIFT))
    }
}

#[cfg(test)]
mod test;
