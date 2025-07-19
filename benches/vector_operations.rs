use amazon_rose_forest::core::vector::Vector;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_vector_operations(c: &mut Criterion) {
    let dimensions = vec![10, 100, 1000, 10000];

    // Benchmark vector creation
    {
        let mut group = c.benchmark_group("vector_creation");
        for &dim in &dimensions {
            group.bench_with_input(BenchmarkId::new("zeros", dim), &dim, |b, &dim| {
                b.iter(|| Vector::zeros(black_box(dim)))
            });

            group.bench_with_input(BenchmarkId::new("random", dim), &dim, |b, &dim| {
                b.iter(|| Vector::random(black_box(dim)))
            });
        }
        group.finish();
    }

    // Benchmark vector operations
    {
        let mut group = c.benchmark_group("vector_operations");
        for &dim in &dimensions {
            let v1 = Vector::random(dim);
            let v2 = Vector::random(dim);

            group.bench_with_input(BenchmarkId::new("dot_product", dim), &dim, |b, _| {
                b.iter(|| black_box(&v1).dot(black_box(&v2)))
            });

            group.bench_with_input(BenchmarkId::new("euclidean_distance", dim), &dim, |b, _| {
                b.iter(|| black_box(&v1).euclidean_distance(black_box(&v2)))
            });

            group.bench_with_input(BenchmarkId::new("cosine_similarity", dim), &dim, |b, _| {
                b.iter(|| black_box(&v1).cosine_similarity(black_box(&v2)))
            });

            group.bench_with_input(BenchmarkId::new("normalize", dim), &dim, |b, _| {
                b.iter(|| black_box(&v1).normalize())
            });
        }
        group.finish();
    }

    // Benchmark batch operations
    {
        let mut group = c.benchmark_group("batch_operations");
        for &dim in &dimensions {
            let v1 = Vector::random(dim);
            let batch_size = 100;
            let others: Vec<Vector> = (0..batch_size).map(|_| Vector::random(dim)).collect();

            group.bench_with_input(
                BenchmarkId::new(format!("batch_euclidean_{}dim", dim), batch_size),
                &batch_size,
                |b, _| b.iter(|| black_box(&v1).batch_euclidean_distance(black_box(&others))),
            );

            group.bench_with_input(
                BenchmarkId::new(format!("batch_cosine_{}dim", dim), batch_size),
                &batch_size,
                |b, _| b.iter(|| black_box(&v1).batch_cosine_similarity(black_box(&others))),
            );
        }
        group.finish();
    }
}

criterion_group!(benches, bench_vector_operations);
criterion_main!(benches);
