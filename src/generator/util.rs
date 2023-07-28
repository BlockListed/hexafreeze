use crate::generator::nano::Nanosecond;
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
