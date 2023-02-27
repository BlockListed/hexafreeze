use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

fn setup_logging() -> impl Drop {
    let (flame, _guard) = tracing_flame::FlameLayer::with_file("./tracing.flame").unwrap();

    let subscriber = tracing_subscriber::Registry::default().with(flame);
    let _ = tracing::subscriber::set_global_default(subscriber);
    _guard
}

fn setup_generator() -> hexafreeze::Generator {
    hexafreeze::Generator::new(0, *hexafreeze::DEFAULT_EPOCH).unwrap()
}

fn setup_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .unwrap()
}

pub fn generate(c: &mut Criterion) {
    setup_logging();
    let gen = setup_generator();

    c.bench_function("generate", |b| {
        let rt = setup_runtime();
        b.to_async(rt).iter(|| black_box(gen.generate()));
    });
}

pub fn fast(c: &mut Criterion) {
    setup_logging();

    let gen = setup_generator();

    c.bench_function("fast", |b| {
        let rt = setup_runtime();
        b.to_async(rt).iter(|| async {
            for _ in 0..10000 {
                black_box(gen.generate().await.unwrap());
            }
        })
    });
}

criterion_group!(benches, generate, fast);
criterion_main!(benches);
