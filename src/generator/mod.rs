use chrono::{DateTime, Utc};
use log::error;
use tokio::sync::{mpsc, oneshot};

use std::fmt::Display;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Waker, Poll};
use std::future::Future;
use std::thread;
use std::mem::replace;
use std::time;

use crate::util::constants;

mod util;

#[derive(Debug)]
#[non_exhaustive]
pub enum GeneratorError {
    EpochInTheFuture,
    EpochTooFarInThePast,
    IncrementBiggerThanSixtyFourBitSignedLimit,
    NodeIdToBig,
    ItsTheFuture,
    Other(String),
}

impl Display for GeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneratorError::EpochInTheFuture => write!(f, "Epoch is in the future!"),
            GeneratorError::EpochTooFarInThePast => write!(f, "Epoch is more than 2**41-1 milliseconds in the past."),
            GeneratorError::IncrementBiggerThanSixtyFourBitSignedLimit => write!(f, "You are trying to generate more than 2**63-1 ids, which is sadly impossible."),
            GeneratorError::NodeIdToBig => write!(f, "Node id is bigger than 2**10-1 (1024)."),
            GeneratorError::ItsTheFuture => write!(f, "It's literally been more than 200k years since the invention of rust, please and this library is definetely deprecated now."),
            GeneratorError::Other(x) => write!(f, "{}", x),
        }
    }
}

pub type GeneratorResponse = Result<i64, GeneratorError>;

#[derive(Clone)]
pub struct Generator {
    generator: InternalGeneratorHandle,
}

impl Generator {
    /// Create a new generator with the default size request buffer (16).
    ///
    /// If the request buffer is filled, then new requests will wait, but still come through.
    /// In practice it should probably never fill though.
    /// 
    /// # Errors
    /// * `node_id` is bigger than 2**10-1 (1024).
    /// * Epoch is in the future
    /// * Epoch is more than 2**41-1 milliseconds in the past
    /// * Not an error, but it will cause issues if the time is later than ~ 290k years AD.
    pub fn new(node_id: i64, epoch: DateTime<Utc>) -> Result<Self, GeneratorError> {
        Self::new_custom_buffer_size(node_id, epoch, constants::DEFAULT_BUFFER_SIZE)
    }

    /// Create a new generator with a custom size for the request buffer. (Advanced users only)
    ///
    /// If the request buffer is filled, then new requests will wait, but still come through.
    /// In practice it should probably never fill though.
    /// 
    /// # Errors
    /// * `node_id` is bigger than 2**10-1 (1024).
    /// * Epoch is in the future
    /// * Epoch is more than 2**41-1 milliseconds in the past
    /// * Not an error, but it will cause issues if the time is later than ~ 290k years AD.
    pub fn new_custom_buffer_size(node_id: i64, epoch: DateTime<Utc>, req_buffer_size: usize) -> Result<Self, GeneratorError> {
        let (req_sender, req_receiver) = mpsc::channel::<GeneratorRequest>(req_buffer_size);

        let epoch_micros = epoch.timestamp_micros();
        let handle = InternalGenerator::new(node_id, epoch_micros, req_receiver, req_sender)?;

        return Ok(Self {
            generator: handle,
        })
    }
    
    /// Generate a new [i64] id.
    /// 
    /// # Errors
    /// * Generator thread panics
    /// * Generating more than [`i64::MAX`] ids. (in total)
    /// * Epoch is in the future.
    /// * Epoch is more than 2**41-1 milliseconds in the past.
    /// * The time is later than ~290k years AD.
    pub async fn generate(&self) -> Result<i64, GeneratorError> {
        let increment = self.generator.get_new_increment()?;
        let (tx, rx) = oneshot::channel::<GeneratorResponse>();
        let w = Arc::new(Mutex::new(None));
        let request = GeneratorRequest {
            req_id: increment,
            return_channel: tx,
            waker: Arc::clone(&w),
        };
        if let Err(x) = self.generator.request_channel.send(request).await {
            return Err(GeneratorError::Other(x.to_string()))
        }

        let future_id = GeneratorIdFuture {
            return_channel: rx,
            waker: w,
        };

        return future_id.await
    }
}

struct GeneratorRequest {
    pub req_id: i64,
    // Why in gods name did I decide on thread to thread communication.
    pub return_channel: oneshot::Sender<GeneratorResponse>,
    pub waker: Arc<Mutex<Option<Waker>>>,
}

struct GeneratorIdFuture {
    // Why in gods name did I decide on thread to thread communication.
    pub return_channel: oneshot::Receiver<GeneratorResponse>,
    pub waker: Arc<Mutex<Option<Waker>>>,
}

impl Future for GeneratorIdFuture {
    type Output = Result<i64, GeneratorError>;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        match self.return_channel.try_recv() {
            Ok(x) => Poll::Ready(x),
            Err(x) => {
                match x {
                    oneshot::error::TryRecvError::Empty => {
                        // I do know this may block.
                        // But it will not block for very long
                        // and this is shown in the rust async book.
                        // https://rust-lang.github.io/async-book/02_execution/03_wakeups.html
                        *self.waker.lock().unwrap() = Some(cx.waker().clone());
                        return Poll::Pending
                    },
                    oneshot::error::TryRecvError::Closed => {
                        return Poll::Ready(Err(GeneratorError::Other("Internal generator seems to have panicked.".to_string())))
                    }
                }
            } 
        }
    }
}

struct InternalGenerator {
    node_id: i64,
    epoch_micros: i64,

    increment: Arc<AtomicI64>,
    last_reset_time_micros: i64,
    distribute_sleep: bool,
    req_receiver: mpsc::Receiver<GeneratorRequest>,
}

#[derive(Clone)]
struct InternalGeneratorHandle {
    increment: Arc<AtomicI64>,
    pub request_channel: mpsc::Sender<GeneratorRequest>,
}

impl InternalGeneratorHandle {
    pub fn get_new_increment(&self) -> Result<i64, GeneratorError> {
        // SeqCst cause this shit is too damn confusing and
        // the limiting factor to performance is inherent to snowflakes.
        let increment = self.increment.fetch_add(1, Ordering::SeqCst);
        if increment == i64::MAX {
            return Err(GeneratorError::IncrementBiggerThanSixtyFourBitSignedLimit);
        }

        return Ok(increment);
    }
}

impl InternalGenerator {
    fn check_creation_parameters(node_id: i64, epoch_micros: i64) -> Result<(), GeneratorError> {
        if node_id > constants::MAX_NODE_ID {
            return Err(GeneratorError::NodeIdToBig)
        }

        let now = util::now_micros()?;

        if now - epoch_micros < 0 {
            return Err(GeneratorError::EpochInTheFuture)
        }

        if (now - epoch_micros) / 1000 < constants::MAX_TIMESTAMP_MILLIS {
            return Err(GeneratorError::EpochTooFarInThePast)
        }

        Ok(())
    }

    // Returning the generator itself is a bad idea.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(node_id: i64, epoch_micros: i64, req_receiver: mpsc::Receiver<GeneratorRequest>, req_sender: mpsc::Sender<GeneratorRequest>) -> Result<InternalGeneratorHandle, GeneratorError> {
        Self::check_creation_parameters(node_id, epoch_micros)?;

        let data = Self {
            node_id,
            epoch_micros,
            increment: Arc::new(AtomicI64::new(0)),
            last_reset_time_micros: 0,
            distribute_sleep: false,
            req_receiver,
        };

        let r = InternalGeneratorHandle {
            increment: Arc::clone(&data.increment),
            request_channel: req_sender,
        };

        data.spawn();

        return Ok(r)
    }

    fn spawn(self) {
        thread::spawn(move || {
            self.main_loop();
        });
    }

    // Main loop for the generator Thread
    fn main_loop(mut self) {
        while let Some(r) = self.req_receiver.blocking_recv() {
            let resp = self.generate(&r);

            if r.return_channel.send(resp).is_err() {
                error!("Receiving end of ID hung up before we could generate!");
            }
            if let Some(w) = (*r.waker.lock().unwrap()).take() {
                w.wake();
            }
        }
    }

    // Functions for generation
    fn generate(&mut self, r: &GeneratorRequest) -> GeneratorResponse {
        let seq = r.req_id % constants::RESET_INCREMENT;
        self.distribute_sleep();
            
        let timestamp_micros = self.get_timestamp(seq)?;

        return self.create_id(timestamp_micros, seq)
    }

    // This function will distribute sleep over every generate to avoid spikes in generation time.
    // It only runs when you request more than 2**10-1 ids per millisecond.
    fn distribute_sleep(&self) {
        if self.distribute_sleep {
            thread::sleep(constants::DISTRIBUTED_SLEEP_TIME);
        }
    }

    fn get_timestamp(&mut self, seq: i64) -> Result<i64, GeneratorError> {
        let mut now = util::now_micros()?;
        if seq == 0 {
            let (should_sleep, sleep_time) = self.check_min_reset_time_and_sleep(now);
            self.distribute_sleep = should_sleep;
            // Make sure `now` is actually the (expected) current time.
            now += sleep_time;
        }

        return Ok(now);
    }

    fn check_min_reset_time_and_sleep(&mut self, now: i64) -> (bool, i64) {
        let last_reset_micros = replace(&mut self.last_reset_time_micros, now);

        // Earliest time, where it is acceptable to not sleep.
        let earliest_acceptable = last_reset_micros + constants::MINIMUM_TIME_BETWEEN_RESET_MICROS;

        let sleep_time = earliest_acceptable - now;
        if sleep_time > 0 {
            // This is ok, because `sleep_time` is always above 0 AND
            // `sleep_time` is always smaller than `constants::MINIMUM_TIME_BETWEEN_RESET_MICROS`,
            // assuming, that `last_reset_micros` is in the past.
            #[allow(clippy::cast_sign_loss)]
            thread::sleep(time::Duration::from_micros(sleep_time as u64));
            return (true, sleep_time)
        }
        return (false, 0)
    }

    fn create_id(&self, ts_micros: i64, seq: i64) -> GeneratorResponse {
        let since_epoch_millis = (ts_micros - self.epoch_micros).div_euclid(1000);
        if since_epoch_millis < 0 {
            return Err(GeneratorError::EpochInTheFuture)
        };
        if since_epoch_millis > constants::MAX_TIMESTAMP_MILLIS {
            return Err(GeneratorError::EpochTooFarInThePast)
        };
        return Ok(util::create_snowflake(since_epoch_millis, self.node_id, seq))
    }
}

#[cfg(test)]
mod test;