use criterion::{black_box, criterion_group, criterion_main, Criterion};
use scratchpad::Config;

fn bench_config_creation(c: &mut Criterion) {
    c.bench_function("config_default", |b| b.iter(|| Config::default()));
}

fn bench_config_serialization(c: &mut Criterion) {
    let config = Config::default();

    c.bench_function("config_to_toml", |b| {
        b.iter(|| toml::to_string(&black_box(&config)))
    });

    c.bench_function("config_to_toml_pretty", |b| {
        b.iter(|| toml::to_string_pretty(&black_box(&config)))
    });

    let toml_str = toml::to_string(&config).unwrap();
    c.bench_function("config_from_toml", |b| {
        b.iter(|| toml::from_str::<Config>(black_box(&toml_str)))
    });
}

fn bench_config_json_serialization(c: &mut Criterion) {
    let config = Config::default();

    c.bench_function("config_to_json", |b| {
        b.iter(|| serde_json::to_string(&black_box(&config)))
    });

    let json_str = serde_json::to_string(&config).unwrap();
    c.bench_function("config_from_json", |b| {
        b.iter(|| serde_json::from_str::<Config>(black_box(&json_str)))
    });
}

fn bench_config_access(c: &mut Criterion) {
    let config = Config::default();

    c.bench_function("config_server_access", |b| {
        b.iter(|| {
            let _ = black_box(&config).server.port;
        })
    });

    c.bench_function("config_docker_access", |b| {
        b.iter(|| {
            let _ = black_box(&config).docker.socket.clone();
        })
    });

    c.bench_function("config_debug_format", |b| {
        b.iter(|| format!("{:?}", black_box(&config)))
    });
}

criterion_group!(
    benches,
    bench_config_creation,
    bench_config_serialization,
    bench_config_json_serialization,
    bench_config_access
);
criterion_main!(benches);
