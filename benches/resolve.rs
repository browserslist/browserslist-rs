use browserslist::{resolve, Opts};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn resolve_defaults_not_dead(c: &mut Criterion) {
    c.bench_function("resolve 'defaults, not dead'", |b| {
        b.iter(|| {
            resolve(
                black_box(vec!["defaults, not dead"]),
                &black_box(Opts::new()),
            )
        })
    });
}

pub fn resolve_usage(c: &mut Criterion) {
    c.bench_function("resolve '> 0.5%'", |b| {
        b.iter(|| resolve(black_box(vec!["> 0.5%"]), &black_box(Opts::new())))
    });
}

pub fn resolve_cover(c: &mut Criterion) {
    c.bench_function("resolve 'cover 99%'", |b| {
        b.iter(|| resolve(black_box(vec!["cover 99%"]), &black_box(Opts::new())))
    });
}

pub fn resolve_electron(c: &mut Criterion) {
    c.bench_function("resolve 'electron >= 10'", |b| {
        b.iter(|| resolve(black_box(vec!["electron >= 10"]), &black_box(Opts::new())))
    });
}

pub fn resolve_node(c: &mut Criterion) {
    c.bench_function("resolve 'node >= 8'", |b| {
        b.iter(|| resolve(black_box(vec!["node >= 8"]), &black_box(Opts::new())))
    });
}

pub fn resolve_browser_features(c: &mut Criterion) {
    c.bench_function("resolve 'supports es6-module'", |b| {
        b.iter(|| {
            resolve(
                black_box(vec!["supports es6-module"]),
                &black_box(Opts::new()),
            )
        })
    });
}

criterion_group!(
    benches,
    resolve_defaults_not_dead,
    resolve_usage,
    resolve_cover,
    resolve_electron,
    resolve_node,
    resolve_browser_features
);
criterion_main!(benches);
