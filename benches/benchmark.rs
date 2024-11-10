use criterion::{criterion_group, criterion_main, Criterion};
use geo_types::{line_string, polygon, Geometry, Line};
use klippa::ClipRect;

// clips line at single point
fn lineclip() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = Line::new((0.0, 0.0), (5.0, 5.0));
    rect.clip(&Geometry::Line(g)).unwrap();
}

// clips line at multiple points
fn lineclip_multi() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = Line::new((-1.0, -1.0), (5.0, 5.0));
    rect.clip(&Geometry::Line(g)).unwrap();
}

fn linestringclip() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = line_string![
        (x: -1.0, y: 2.0), (x: 1.0, y:2.0),
        (x: 1.0, y: 2.0), (x: 5.0, y: 2.0)
    ];
    rect.clip(&Geometry::LineString(g)).unwrap();
}

fn polyclip() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = polygon![(x: 1.0, y: 1.0), (x: 5.0, y: 5.0)];
    rect.clip(&Geometry::Polygon(g)).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("lineclip", |b| b.iter(|| lineclip()));
    c.bench_function("lineclip_multi", |b| b.iter(|| lineclip_multi()));
    c.bench_function("linestringclip", |b| b.iter(|| linestringclip()));
    c.bench_function("polyclip", |b| b.iter(|| polyclip()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
