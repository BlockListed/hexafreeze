use crate::util::constants;

pub fn create_snowflake(ts: i64, node_id: i64, seq: i64) -> i64 {
    return (ts << constants::TIMESTAMP_SHIFT)
        | (node_id << constants::INSTANCE_SHIFT)
        | (seq << constants::SEQUENCE_SHIFT);
}

pub fn now_micros() -> Result<i64, super::GeneratorError> {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Bruh the clock says it's before 1970.")
        .as_micros();
    match t.try_into() {
        Ok(x) => Ok(x),
        Err(_) => Err(super::GeneratorError::ItsTheFuture),
    }
}
