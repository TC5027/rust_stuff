use challenge_zama::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group_linear_combination = c.benchmark_group("linear combination comparison");

    for size in [1_000usize, 2_000, 3_000].iter() {
        group_linear_combination.bench_with_input(
            BenchmarkId::new("without Rayon", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut matrix = Matrix::new(10_000, 1, &vec![1; 10_000]);
                    let weight = Matrix::new(size, 10_000, &vec![1; size * 10_000]);
                    let bias = Matrix::new(size, 1, &vec![1; size]);
                    matrix.linear_combination(weight, bias);
                })
            },
        );

        group_linear_combination.bench_with_input(
            BenchmarkId::new("with Rayon", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut matrix = Matrix::new(10_000, 1, &vec![1; 10_000]);
                    let weight = Matrix::new(size, 10_000, &vec![1; size * 10_000]);
                    let bias = Matrix::new(size, 1, &vec![1; size]);
                    matrix.par_linear_combination(weight, bias);
                })
            },
        );
    }

    group_linear_combination.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
