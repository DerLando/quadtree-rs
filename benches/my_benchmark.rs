use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quadtree::{Point, QuadTree, Rectangle};

fn insert_point_in_quadtree(pt: Point) {
    let mut quadtree = QuadTree::new_bounded(&Rectangle::new((0.0, 0.0), 1000.0, 1000.0));
    quadtree.insert(0u8, pt);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("pt insert 1", |b| {
        b.iter(|| insert_point_in_quadtree(black_box(Point::new(20.0, 50.0))))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
