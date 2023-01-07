use chrono::prelude::*;

#[test]
pub fn new_generator() {
    let should_ok = super::Generator::new(0, *crate::DEFAULT_EPOCH);

    assert!(should_ok.is_ok());
}

#[test]
pub fn new_generator_node_id() {
    let should_error = super::Generator::new(1058, *crate::DEFAULT_EPOCH);
    let should_ok = super::Generator::new(830, *crate::DEFAULT_EPOCH);

    assert!(should_error.is_err());

    assert!(should_ok.is_ok());
}

#[test]
pub fn new_generator_epoch() {
    let should_error_past = super::Generator::new(
        0,
        DateTime::from_utc(
            NaiveDate::from_ymd_opt(1950, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            Utc,
        ),
    );
    assert_eq!(
        should_error_past.err().unwrap(),
        super::GeneratorError::EpochTooFarInThePast
    );

    // this is hopefully gonna stay in the future
    let should_error_future = super::Generator::new(
        0,
        DateTime::from_utc(
            NaiveDate::from_ymd_opt(200000, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            Utc,
        ),
    );
    assert_eq!(
        should_error_future.err().unwrap(),
        super::GeneratorError::EpochInTheFuture
    );
}

#[tokio::test]
pub async fn generate_id() {
    let generator = super::Generator::new(0, *crate::DEFAULT_EPOCH).unwrap();

    let id = generator.generate().await.unwrap();

    assert_ne!(id, 0);
}

#[tokio::test]
pub async fn duplicate_generation() {
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::sync::Arc;
    let d_map: Arc<dashmap::DashSet<i64>> = Arc::new(dashmap::DashSet::new());
    let count = Arc::new(AtomicI64::new(0));
    let gen = Arc::new(super::Generator::new(0, *crate::DEFAULT_EPOCH).unwrap());

    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::with_capacity(num_cpus::get());

    for _ in 0..num_cpus::get() {
        let m = Arc::clone(&d_map);
        let c = Arc::clone(&count);
        let g = Arc::clone(&gen);
        let h = tokio::spawn(async move {
            while c.fetch_add(1, Ordering::AcqRel) < 20480 {
                let id = g.generate().await.unwrap();
                assert!(m.get(&id).is_none());
                m.insert(id);
            }
        });
        handles.push(h);
    }

    for h in handles {
        h.await.unwrap();
    }
}
