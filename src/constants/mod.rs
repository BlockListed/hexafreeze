#![allow(dead_code)]

use once_cell::sync::Lazy;
use crate::generator::nano::Time;
use std::time::Duration;
use uom::si::time::millisecond;

macro_rules! static_time {
    ($name:ident, $t:ty, $v:expr) => {
        pub static $name: Lazy<Time> = Lazy::new(|| Time::new::<$t>($v));
    }
}

// i64, because it operates on another i64,
// so conversion would be necessary.
pub const RESET_INCREMENT: i64 = 1 << 12;

// For snowflake creation (bits)
pub const TIMESTAMP_LENGTH: isize = 41;
pub const INSTANCE_LENGTH: isize = 10;
pub const SEQUENCE_LENGTH: isize = 12;

pub const TIMESTAMP_SHIFT: isize = INSTANCE_LENGTH + SEQUENCE_LENGTH;
pub const INSTANCE_SHIFT: isize = SEQUENCE_LENGTH;
pub const SEQUENCE_SHIFT: isize = 0;

pub const DISTRIBUTED_SLEEP_TIME: Duration =
    Duration::from_nanos(10u64.pow(6) / RESET_INCREMENT as u64);

// Snowflake constants
// Function, since this is very cheap.
static_time!(MAX_TIMESTAMP, millisecond, (1 << 41) - 1);

pub const MAX_NODE_ID: i64 = (1 << 10) - 1;

static_time!(MINIMUM_TIME_BETWEEN_RESET, millisecond, 1);
static_time!(MILLISECOND, millisecond, 1);

pub const DEFAULT_BUFFER_SIZE: usize = 16;

/// 2020-01-01T00:00:00Z
pub const DEFAULT_EPOCH: i64 = 1_577_833_200_000;
