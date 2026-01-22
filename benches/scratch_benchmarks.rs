use criterion::{black_box, criterion_group, criterion_main, Criterion};
use scratchpad::Scratch;

fn bench_scratch_creation(c: &mut Criterion) {
    c.bench_function("scratch_new", |b| {
        b.iter(|| {
            Scratch::new(
                black_box("test-scratch".to_string()),
                black_box("main".to_string()),
                black_box("default".to_string()),
            )
        })
    });
}

fn bench_scratch_sanitize_name(c: &mut Criterion) {
    let mut group = c.benchmark_group("sanitize_name");

    group.bench_function("simple_name", |b| {
        b.iter(|| Scratch::sanitize_name(black_box("simple-branch")))
    });

    group.bench_function("complex_name", |b| {
        b.iter(|| Scratch::sanitize_name(black_box("feature/my-feature-123-test")))
    });

    group.bench_function("special_chars_name", |b| {
        b.iter(|| Scratch::sanitize_name(black_box("feature/my-feature!!!@@@###---test")))
    });

    group.bench_function("long_name", |b| {
        b.iter(|| {
            let long_name = "a".repeat(1000);
            Scratch::sanitize_name(black_box(&long_name))
        })
    });

    group.bench_function("unicode_name", |b| {
        b.iter(|| Scratch::sanitize_name(black_box("feature/café-résumé")))
    });

    group.finish();
}

fn bench_scratch_serialization(c: &mut Criterion) {
    let scratch = Scratch::new(
        "test-scratch".to_string(),
        "main".to_string(),
        "default".to_string(),
    );

    c.bench_function("scratch_to_json", |b| {
        b.iter(|| serde_json::to_string(&black_box(&scratch)))
    });

    c.bench_function("scratch_to_json_pretty", |b| {
        b.iter(|| serde_json::to_string_pretty(&black_box(&scratch)))
    });

    let json = serde_json::to_string(&scratch).unwrap();
    c.bench_function("scratch_from_json", |b| {
        b.iter(|| serde_json::from_str::<Scratch>(black_box(&json)))
    });
}

fn bench_scratch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("scratch_operations");

    group.bench_function("clone_scratch", |b| {
        let scratch = Scratch::new(
            "test".to_string(),
            "main".to_string(),
            "default".to_string(),
        );
        b.iter(|| black_box(&scratch).clone())
    });

    group.bench_function("scratch_debug_format", |b| {
        let scratch = Scratch::new(
            "test".to_string(),
            "main".to_string(),
            "default".to_string(),
        );
        b.iter(|| format!("{:?}", black_box(&scratch)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_scratch_creation,
    bench_scratch_sanitize_name,
    bench_scratch_serialization,
    bench_scratch_operations
);
criterion_main!(benches);
