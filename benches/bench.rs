use criterion::{black_box, criterion_group, criterion_main, Criterion};

use licht::stepping::{Stepping, Blend};

fn bench_blend(c: &mut Criterion) {
    let blend = Blend { ratio: 0.5, a: 2.0, b: 4.0, step: 5 };
    c.bench_function("blend", |bench| bench.iter(|| blend.calculate(black_box(3201), black_box(7500))));
}
criterion_group!(benches, bench_blend);
criterion_main!(benches);
