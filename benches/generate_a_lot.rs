use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn setup_logging() {
    /* 
    if tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter("TRACE")
        .with_writer(std::fs::File::create("/tmp/benchmark.log").unwrap())
        .try_init()
        .is_ok()
    {};
    */
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
