use criterion::{criterion_group, criterion_main, Criterion};

use std::sync::Arc;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("genemerate", |b| {
        let genemerator = Arc::new(hexafreeze::Generator::new(0, *hexafreeze::DEFAULT_EPOCH).unwrap());
        let r = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            r.block_on(genemerator.generate()).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
