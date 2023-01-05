use std::thread::sleep;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use std::sync::atomic;

use log::trace;

// i64, because it operates on another i64,
// so conversion would be necessary.
const RESET_INCREMENT: i64 = 1 << 12;

// For snowflake creation (bits)
#[allow(dead_code)]
const TIMESTAMP_LENGTH: isize = 41;
const INSTANCE_LENGTH: isize = 10;
const SEQUENCE_LENGTH: isize = 12;

const TIMESTAMP_SHIFT: isize = INSTANCE_LENGTH + SEQUENCE_LENGTH;
const INSTANCE_SHIFT: isize = SEQUENCE_LENGTH;
const SEQUENCE_SHIFT: isize = 0;

const DISTRIBUTED_SLEEP_TIME: Duration =
    Duration::from_nanos((10u64.pow(9) / RESET_INCREMENT as u64) as u64);

// Snowflake constants
const MAX_TIMESTAMP_MILLIS: i64 = (1 << 41) - 1;
const MAX_NODE_ID: i64 = (1 << 10) - 1;

pub struct SnowflakeGenerator {
    epoch: DateTime<Utc>,
    node_id: i64,
    // Last time `increment % RESET_INCREMENT == 0`
    last_reset_timestamp_micros: atomic::AtomicI64,
    increment: atomic::AtomicI64,
    // Sleep every generation iteration to avoid latency spikes
    distributed_sleep: atomic::AtomicBool,
    sleep_lock: atomic::AtomicBool,
}

#[derive(Debug)]
pub enum SnowflakeError {
    EpochToLongAgo,
    EpochInTheFuture,
    InstanceToBig,
}

impl SnowflakeGenerator {
    /// Is a const function, so can be used with static without something like lazy_static
    pub const fn new(epoch: DateTime<Utc>, node_id: i64) -> Result<Self, SnowflakeError> {
        if node_id > MAX_NODE_ID {
            return Err(SnowflakeError::InstanceToBig);
        }
        return Ok(Self {
            epoch,
            node_id,
            last_reset_timestamp_micros: atomic::AtomicI64::new(0),
            increment: atomic::AtomicI64::new(0),
            distributed_sleep: atomic::AtomicBool::new(false),
            sleep_lock: atomic::AtomicBool::new(false),
        });
    }

    pub fn get_increment(&self) -> i64 {
        self.increment.load(atomic::Ordering::Acquire)
    }

    /// Returns error, if difference between epoch and now is more than 41 bits long or if the epoch is in the future
    pub fn generate(&self) -> Result<i64, SnowflakeError> {
        let start = Instant::now();
        // Spin on the sleep lock
        while self.sleep_lock.load(atomic::Ordering::Acquire) {};
        let (sequence, ts) = self.get_sequence();

        let id = create_snowflake(
            (ts / 1000) - self.epoch.timestamp_millis(),
            self.node_id,
            sequence,
        )?;

        // If the generator is set to distribute sleep this will distribute
        // the sleep across all generates, until the rate of generation slows down.
        // Note: distributed_sleep only get evaluated at a sequence reset. (Every 4096 ids generated)
        self.distribute_sleep(start);
        return Ok(id);
    }

    // Second return is in micros
    // Return sequence and then micros
    fn get_sequence(&self) -> (i64, i64) {
        let now = current_timestamp_micros();

        let counter = self.increment.fetch_add(1, atomic::Ordering::SeqCst);
        let sequence = counter % RESET_INCREMENT;
        trace!("Counter at {} for generation! Seq: {}", counter, sequence);
        let mut delta: i64 = 0;
        if sequence == 0 {
            // FIXME: think about whether the ordering can/needs to be changed
            // If this has a race condition, then you're fucked anyway,
            // because if you're generating 4096 ids in different threads,
            // then you should use something else
            let last_time = self
                .last_reset_timestamp_micros
                .swap(now, atomic::Ordering::AcqRel);

            // FIXME: Something here is fucking broken;
            delta = now - (last_time + 1000);
            if delta < 0 {
                self.distributed_sleep
                    .store(true, atomic::Ordering::Release);
                self.sleep_lock.store(true, atomic::Ordering::Release);
                sleep(Duration::from_micros(delta.abs() as u64));
                self.sleep_lock.store(false, atomic::Ordering::Release);
            } else {
                self.distributed_sleep
                    .store(false, atomic::Ordering::Release);
            }
        }

        // If we have slept, then return time after sleep
        return (sequence, if delta < 0 { now + delta.abs() } else { now });
    }

    fn distribute_sleep(&self, start: Instant) {
        if self.distributed_sleep.load(atomic::Ordering::Relaxed) {
            if let Some(x) = DISTRIBUTED_SLEEP_TIME.checked_sub(start.elapsed()) {
                sleep(x);
            }
        }
    }
}

fn create_snowflake(ts: i64, id: i64, seq: i64) -> Result<i64, SnowflakeError> {
    if ts > MAX_TIMESTAMP_MILLIS {
        return Err(SnowflakeError::EpochToLongAgo);
    }
    return Ok((ts << TIMESTAMP_SHIFT) | (id << INSTANCE_SHIFT) | (seq << SEQUENCE_SHIFT));
}

#[cfg(test)]
mod test {
    use crate::SnowflakeGenerator;
    use std::str::FromStr;

    #[test]
    fn test_duplicate_prevention() {
        use dashmap::DashMap;
        use num_cpus::get;
        use std::sync::atomic;
        use std::sync::Arc;
        use std::thread;
        env_logger::init();
        let genemerator = Arc::new(
            SnowflakeGenerator::new(chrono::DateTime::from_str("2020-1-1T00:00:00Z").unwrap(), 1)
                .unwrap(),
        );
        let duplicate_map: Arc<DashMap<i64, ()>> = Arc::new(DashMap::new());

        const ITERATIONS: u64 = 1_000_000;
        let counter: Arc<atomic::AtomicU64> = Arc::new(atomic::AtomicU64::new(0));

        thread::scope(|s| {
            for i in 0..get() {
                let g = Arc::clone(&genemerator);
                let m = Arc::clone(&duplicate_map);
                let c = Arc::clone(&counter);
                s.spawn(move || {
                    loop {
                        let iterations = c.fetch_add(1, atomic::Ordering::AcqRel);
                        if iterations >= ITERATIONS {
                            break;
                        }
                        if iterations % 10_000 == 0 {
                            eprintln!("{}", iterations);
                        }
                        let id = g.generate().unwrap();
                        if m.get(&id).is_some() {
                            panic!("Id {} is a duplicate! Inc: {}", id, g.get_increment());
                        }
                        m.insert(id, ());
                    }
                    return
                });
                eprintln!("Thread {} spawned", i);
            }
        });
    }
}

#[inline(always)]
fn current_timestamp_micros() -> i64 {
    Utc::now().timestamp_micros()
}
