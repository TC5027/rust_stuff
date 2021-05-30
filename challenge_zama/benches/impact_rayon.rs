use challenge_zama::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group_linear_combination = c.benchmark_group("linear combination comparison");

    for &size in [1_000usize, 2_000, 3_000].iter() {
        let weight = Matrix::new(size, size, &vec![1; size * size]);
        let bias = Matrix::new(1, size, &vec![1; size]);

        let mut matrix = Matrix::new(1, size, &vec![1; size]);
        group_linear_combination.bench_with_input(
            BenchmarkId::new("without Rayon", size),
            &size,
            |b, _size| {
                b.iter(|| {
                    matrix.linear_combination(&weight, &bias);
                })
            },
        );

        let mut matrix = Matrix::new(1, size, &vec![1; size]);
        group_linear_combination.bench_with_input(
            BenchmarkId::new("with Rayon", size),
            &size,
            |b, _size| {
                b.iter(|| {
                    matrix.par_linear_combination(&weight, &bias);
                })
            },
        );
    }

    group_linear_combination.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
