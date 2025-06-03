use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use amazon_rose_forest::sharding::vector_index::{VectorIndex, DistanceMetric};
use amazon_rose_forest::core::vector::Vector;
use std::collections::HashMap;

fn bench_vector_index(c: &mut Criterion) {
    // Setup test vectors
    let dimensions = 128;
    let vector_count = 10000;
    let test_vectors: Vec<Vector> = (0..vector_count)
        .map(|_| Vector::random_normal(dimensions, 0.0, 1.0))
        .collect();
    
    // Setup query vectors
    let query_count = 5;
    let query_vectors: Vec<Vector> = (0..query_count)
        .map(|_| Vector::random_normal(dimensions, 0.0, 1.0))
        .collect();

    // Test different distance metrics
    let metrics = [
        DistanceMetric::Euclidean,
        DistanceMetric::Cosine,
        DistanceMetric::Manhattan,
        DistanceMetric::Hamming,
    ];
    
    // Setup benchmark for index creation and insertion
    {
        let mut group = c.benchmark_group("vector_index_creation");
        for &metric in &metrics {
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", metric), vector_count),
                &vector_count,
                |b, _| {
                    b.iter_with_setup(
                        || {
                            // Create a new index for each iteration
                            let index = VectorIndex::new("bench_index", dimensions, metric, None);
                            (index, test_vectors.clone())
                        },
                        |(index, vectors)| {
                            // Use tokio for async operations in benchmarks
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            
                            rt.block_on(async {
                                for vector in vectors {
                                    let _ = index.add(vector, None).await.unwrap();
                                }
                            });
                        },
                    );
                },
            );
        }
        group.finish();
    }
    
    // Setup benchmark for vector search
    {
        let mut group = c.benchmark_group("vector_index_search");
        for &metric in &metrics {
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", metric), vector_count),
                &vector_count,
                |b, _| {
                    // Setup the index with test vectors
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let index = VectorIndex::new("bench_index", dimensions, metric, None);
                    
                    rt.block_on(async {
                        for vector in &test_vectors {
                            let _ = index.add(vector.clone(), None).await.unwrap();
                        }
                    });
                    
                    b.iter_with_setup(
                        || query_vectors[0].clone(), // Use first query vector for benchmarking
                        |query| {
                            rt.block_on(async {
                                let _ = index.search(&query, 10).await.unwrap();
                            });
                        },
                    );
                },
            );
        }
        group.finish();
    }
    
    // Setup benchmark for batch search operations
    {
        let mut group = c.benchmark_group("vector_index_batch_search");
        group.bench_with_input(
            BenchmarkId::new("batch_size", query_count),
            &query_count,
            |b, _| {
                // Setup the index with test vectors
                let rt = tokio::runtime::Runtime::new().unwrap();
                let index = VectorIndex::new("bench_index", dimensions, DistanceMetric::Cosine, None);
                
                rt.block_on(async {
                    for vector in &test_vectors {
                        let _ = index.add(vector.clone(), None).await.unwrap();
                    }
                });
                
                b.iter(|| {
                    rt.block_on(async {
                        let mut results = Vec::new();
                        for query in &query_vectors {
                            let r = index.search(query, 10).await.unwrap();
                            results.push(r);
                        }
                        results
                    });
                });
            },
        );
        group.finish();
    }
}

criterion_group!(benches, bench_vector_index);
criterion_main!(benches);