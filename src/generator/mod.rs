use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use tokio::time::Instant;

mod checks;
mod util;

pub mod nano;

use nano::{Millisecond, Nanosecond};

use crate::HexaFreezeError;
use crate::{constants, error::HexaFreezeResult};

#[derive(Clone)]
pub struct Generator {
    epoch: Nanosecond,
    node_id: i64,
    increment: Arc<Mutex<i64>>,

    last_reset_millis: Arc<[AtomicI64; 4096]>,
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
    /// * When `node_id` is bigger than 1023
    /// * When the epoch_millis is more than ~69 years ago.
    /// * When the epoch_millis is in the future.
    // Ok since it it a string literal and this function is unit tested to not panic.
    #[allow(clippy::missing_panics_doc)]
    pub fn new(node_id: i64, epoch: DateTime<Utc>) -> HexaFreezeResult<Self> {
        let epoch = Nanosecond::from_millis(epoch.timestamp_millis());
        checks::check_node_id(node_id)?;
        checks::check_epoch(epoch)?;

        const ATOMIC_I64_ZERO: AtomicI64 = AtomicI64::new(0);
        Ok(Self {
            epoch,
            node_id,
            increment: Arc::new(Mutex::new(0)),
            last_reset_millis: Arc::new([ATOMIC_I64_ZERO; 4096]),
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
    /// Since it only errors when:
    /// * the epoch is more than ~69 years ago.
    /// * you have generated more than `9_223_372_036_854_775_807` ids. (In total, for this generator)
    /// * your clock jumps backward in time a significant amount.
    pub async fn generate(&self) -> HexaFreezeResult<i64> {
        let start = Instant::now();
        let mut i = self.increment.lock().await;
        self.distribute_sleep(start).await;
        let (seq, now) = self.get_sequence(i.deref_mut()).await?;
        drop(i);

        self.create_id(now, seq)
    }

    async fn distribute_sleep(&self, start: Instant) {
        if self.distribute_sleep.load(Ordering::Relaxed) {
            // Start is required to make sure, that we only sleep the necessary amount of time.
            util::accurate_sleep(
                constants::DISTRIBUTED_SLEEP_TIME
                    .checked_sub(start.elapsed())
                    .unwrap_or(Duration::ZERO),
            )
            .await;
        }
    }

    async fn get_sequence(&self, inc: &mut i64) -> HexaFreezeResult<(i64, Nanosecond)> {
        let now: Millisecond = util::now().into();
        let seq = *inc % constants::RESET_INCREMENT;
        if *inc == i64::MAX {
            return Err(HexaFreezeError::Surpassed64BitLimit);
        }
        *inc += 1;

        let last = Millisecond(self.last_reset_millis[seq as usize].swap(now.0, Ordering::Relaxed));

        tracing::trace!(?now, ?last, seq);
        if now < last {
            return Err(HexaFreezeError::ClockWentBackInTime);
        }

        if last == now {
            tokio::time::sleep(Duration::from_millis(1)).await;
            tracing::debug!(
                "Sleeping, because generator is overloaded. (Rate higher than 4096 IDs/millisecond)"
            );
            self.distribute_sleep.store(true, Ordering::Relaxed);
            tracing::trace!("Enabled distributed sleep!");

            // No .abs(), because we know its bigger than 0
            return Ok((seq, util::now()));
        }

        if self.distribute_sleep.swap(false, Ordering::Relaxed) {
            tracing::trace!("Disabled distributed sleep!");
        }

        Ok((seq, (now + Millisecond(1)).into()))
    }

    fn create_id(&self, now: Nanosecond, seq: i64) -> HexaFreezeResult<i64> {
        // We know, that the epoch cant be in the future, since it's checked at when a generator is created.
        let ts = now - self.epoch;

        if ts > constants::MAX_TIMESTAMP {
            return Err(HexaFreezeError::EpochTooFarInThePast);
        }

        let id = (((ts.into_millis()) as i64) << constants::TIMESTAMP_SHIFT)
            | (self.node_id << constants::INSTANCE_SHIFT)
            | (seq << constants::SEQUENCE_SHIFT);
        Ok(id)
    }
}

#[cfg(test)]
mod test;
