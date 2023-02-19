use chrono::{DateTime, Utc};

#[test]
fn invalid_epoch() {
    assert_eq!(
        crate::Generator::new(
            0,
            DateTime::parse_from_rfc3339("2177-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        )
        .err()
        .unwrap(),
        crate::HexaFreezeError::EpochInTheFuture
    );
    assert_eq!(
        crate::Generator::new(
            0,
            DateTime::parse_from_rfc3339("1930-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        )
        .err()
        .unwrap(),
        crate::HexaFreezeError::EpochTooFarInThePast
    );
}

#[test]
fn node_id_validation() {
    assert!(crate::Generator::new(0, *crate::DEFAULT_EPOCH).is_ok());
    assert!(crate::Generator::new(214, *crate::DEFAULT_EPOCH).is_ok());
    assert!(crate::Generator::new(1026, *crate::DEFAULT_EPOCH).is_err());
}

#[tokio::test]
async fn generation() {
    let gen = crate::Generator::new(0, *crate::DEFAULT_EPOCH).unwrap();

    assert_ne!(gen.generate().await.unwrap(), 0);
}

#[tokio::test]
async fn duplicate_generation_test() {
    use crate::Generator;
    use crate::DEFAULT_EPOCH;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    const ID_COUNT: usize = 4_096_00;

    let generator = Generator::new(10, *DEFAULT_EPOCH).unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let map = Arc::new(dashmap::DashSet::<i64>::with_capacity(ID_COUNT));

    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    for _ in 0..num_cpus::get() {
        let c = Arc::clone(&counter);
        let g = generator.clone();
        let m = Arc::clone(&map);

        handles.push(tokio::spawn(async move {
            while c.fetch_add(1, Ordering::AcqRel) < ID_COUNT {
                let id = g.generate().await.unwrap();
                if !m.insert(id) {
                    panic!("Big Oof a squidoosh squidoodle happened and we created a duplicate ID");
                }
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }
}
