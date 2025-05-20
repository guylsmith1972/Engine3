// benches/intersection_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Use items from our library crate "convex_polygon_intersection"
use convex_polygon_intersection::geometry::{ConvexPolygon, Point2, MAX_VERTICES};
use convex_polygon_intersection::generator::PolygonGenerator;
use convex_polygon_intersection::intersection::ConvexIntersection;
use rand::Rng;

fn create_test_pair(rng: &mut impl Rng) -> (ConvexPolygon, ConvexPolygon) {
    let vertices1 = rng.gen_range(3..=MAX_VERTICES.min(8));
    let radius1 = rng.gen_range(60.0..100.0);
    let poly1 = PolygonGenerator::generate_convex_polygon(0.0, 0.0, radius1, vertices1);

    let vertices2 = rng.gen_range(3..=MAX_VERTICES.min(8));
    let radius2 = rng.gen_range(60.0..100.0);
    let poly2 = PolygonGenerator::generate_convex_polygon(50.0, 0.0, radius2, vertices2); // Slightly offset
    (poly1, poly2)
}

fn intersection_benchmark_fn(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    const NUM_BENCH_PAIRS: usize = 100;
    let mut pairs: Vec<(ConvexPolygon, ConvexPolygon)> = Vec::with_capacity(NUM_BENCH_PAIRS);
    for _ in 0..NUM_BENCH_PAIRS {
        pairs.push(create_test_pair(&mut rng));
    }
    
    let mut group = c.benchmark_group("IntersectionOperations");
    // group.sample_size(10); // For faster feedback during development, default is 100

    group.bench_function("find_intersection_into_100_pairs_reused_result", |b| {
        let mut result_poly = ConvexPolygon::new(); 
        let mut pair_iter = pairs.iter().cycle(); 

        b.iter(|| {
            let (poly1, poly2) = pair_iter.next().unwrap();
            ConvexIntersection::find_intersection_into(
                black_box(poly1), 
                black_box(poly2),
                black_box(&mut result_poly), 
            )
        })
    });
    group.finish();
}

criterion_group!(benches, intersection_benchmark_fn);
criterion_main!(benches);