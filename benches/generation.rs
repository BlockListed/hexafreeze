use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("genemerate", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(benchmark())
        })
    });
}

async fn benchmark() {
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::sync::Arc;

    let genemerator = Arc::new(hexafreeze::Generator::new(0, *hexafreeze::DEFAULT_EPOCH).unwrap());
    let counter = Arc::new(AtomicI64::new(0));

    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    for _ in 0..num_cpus::get() {
        let g = Arc::clone(&genemerator);
        let c = Arc::clone(&counter);
        handles.push(tokio::spawn(async move {
            while c.fetch_add(1, Ordering::AcqRel) < 200 {
                g.generate().await.unwrap();
            }
        }))
    }

    for h in handles {
        h.await.unwrap();
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
