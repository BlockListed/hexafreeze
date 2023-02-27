#![allow(dead_code)]

use crate::generator::nano::Nanosecond;
use std::time::Duration;

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
pub const MAX_TIMESTAMP: Nanosecond = Nanosecond::from_millis((1 << 41) - 1);

pub const MAX_NODE_ID: i64 = (1 << 10) - 1;

pub const MINIMUM_TIME_BETWEEN_RESET: Nanosecond = Nanosecond::from_millis(1);
pub const MILLISECOND: Nanosecond = Nanosecond::from_millis(1);

pub const DEFAULT_BUFFER_SIZE: usize = 16;

/// 2020-01-01T00:00:00Z
pub const DEFAULT_EPOCH: i64 = 1_577_833_200_000;
