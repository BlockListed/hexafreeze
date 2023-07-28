use std::ops::DerefMut;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;

mod checks;
mod util;

pub mod nano;

use nano::{Millisecond, Nanosecond};

use crate::HexaFreezeError;
use crate::{constants, error::HexaFreezeResult};

#[derive(Clone)]
pub struct Generator {
    inner: Arc<Inner>,
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
    /// * When `node_id` is less than 0
    /// * When the `epoch` is more than ~69 years ago.
    /// * When the `epoch` is in the future.
    pub fn new(node_id: i64, epoch: DateTime<Utc>) -> HexaFreezeResult<Self> {
        let epoch = Nanosecond::from_millis(epoch.timestamp_millis());
        checks::check_node_id(node_id)?;
        checks::check_epoch(epoch)?;

        // Fine, since this is only used for list initialization and is an alternative to ´std::sync::atomic::ATOMIC_I64_INIT´.
        #[allow(clippy::declare_interior_mutable_const)]
        const ATOMIC_I64_ZERO: AtomicI64 = AtomicI64::new(0);

        let inner = Arc::new(Inner {
            epoch,
            node_id,
            increment: Default::default(),
            last_reset_millis: [ATOMIC_I64_ZERO; 4096],
        });

        Ok(Self {
            inner,
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
        let mut i = self.inner.increment.lock().await;
        let (seq, now) = self.get_sequence(i.deref_mut()).await?;
        drop(i);

        self.create_id(now, seq)
    }

    async fn get_sequence(&self, inc: &mut i64) -> HexaFreezeResult<(i64, Nanosecond)> {
        let now: Millisecond = util::now().into();
        let seq = *inc % constants::RESET_INCREMENT;
        if *inc == i64::MAX {
            return Err(HexaFreezeError::Surpassed64BitLimit);
        }
        *inc += 1;

        // `seq` will never be negative.
        // It is an i64, so it can be ORed in `create_id` without casting.
        #[allow(clippy::cast_sign_loss)]
        let last = Millisecond(self.inner.last_reset_millis[seq as usize].swap(now.0, Ordering::Relaxed));

        tracing::trace!(?now, ?last, seq);
        if now < last {
            return Err(HexaFreezeError::ClockWentBackInTime);
        }

        if last == now {
            tokio::time::sleep(Duration::from_millis(1)).await;
            tracing::debug!(
                "Sleeping, because generator is overloaded. (Rate higher than 4096 IDs/millisecond)"
            );

            // No .abs(), because we know its bigger than 0
            return Ok((seq, util::now()));
        }

        Ok((seq, (now + Millisecond(1)).into()))
    }

    fn create_id(&self, now: Nanosecond, seq: i64) -> HexaFreezeResult<i64> {
        // We know, that the epoch cant be in the future, since it's checked at when a generator is created.
        let ts = now - self.inner.epoch;

        if ts > constants::MAX_TIMESTAMP {
            return Err(HexaFreezeError::EpochTooFarInThePast);
        }

        let id = ((ts.into_millis()) << constants::TIMESTAMP_SHIFT)
            | (self.inner.node_id << constants::INSTANCE_SHIFT)
            | (seq << constants::SEQUENCE_SHIFT);
        Ok(id)
    }
}

struct Inner {
    epoch: Nanosecond,
    node_id: i64,
    increment: Mutex<i64>,

    last_reset_millis: [AtomicI64; 4096],
}

#[cfg(test)]
mod test;
