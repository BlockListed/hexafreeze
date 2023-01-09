// WARNING
// This is a benchmark and I am putting this in the examples section, because profiling tests is a lot more difficult.

use hexafreeze::Generator;
use hexafreeze::DEFAULT_EPOCH;
use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc,
};

const ID_COUNT: i64 = 4_096_000;

#[tokio::main]
async fn main() {
    let generator = Generator::new(10, *DEFAULT_EPOCH).unwrap();
    let counter = Arc::new(AtomicI64::new(0));

    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    for _ in 0..num_cpus::get() {
        let c = Arc::clone(&counter);
        let g = generator.clone();

        handles.push(tokio::spawn(async move {
            while c.fetch_add(1, Ordering::AcqRel) < ID_COUNT {
                g.generate().await.unwrap();
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }
}
