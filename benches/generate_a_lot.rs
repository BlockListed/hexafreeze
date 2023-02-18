use criterion::{criterion_group, criterion_main, Criterion};

pub fn generate(c: &mut Criterion) {
    let gen = hexafreeze::Generator::new(0, *hexafreeze::DEFAULT_EPOCH).unwrap();

    c.bench_function("generate", move |b| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .build()
            .unwrap();
        b.to_async(rt).iter(|| gen.generate());
    });
}

criterion_group!(benches, generate);
criterion_main!(benches);
