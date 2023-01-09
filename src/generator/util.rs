use chrono::prelude::*;

pub fn now() -> chrono::DateTime<Utc> {
    Utc::now()
}
