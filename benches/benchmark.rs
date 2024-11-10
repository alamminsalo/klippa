use criterion::{criterion_group, criterion_main, Criterion};
use geo::{coord, BooleanOps};
use geo_types::{line_string, polygon, Geometry, Line};
use klippa::ClipRect;

// clips line at single point
fn lineclip_klippa() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = Line::new((0.0, 0.0), (5.0, 5.0));
    rect.clip(&Geometry::Line(g)).unwrap();
}

fn lineclip_geo() {
    let rect = geo::Rect::new(coord! {x: 0., y: 0.}, coord! {x: 4., y: 4.}).to_polygon();
    let g = Line::new((0.0, 0.0), (5.0, 5.0));
    rect.clip(&g.into(), false);
}

fn linestringclip_klippa() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = line_string![
        (x: -1.0, y: 2.0), (x: 1.0, y:2.0),
        (x: 1.0, y: 2.0), (x: 5.0, y: 2.0)
    ];
    rect.clip(&Geometry::MultiLineString(g.into())).unwrap();
}

fn linestringclip_geo() {
    let rect = geo::Rect::new(coord! {x: 0., y: 0.}, coord! {x: 4., y: 4.}).to_polygon();
    let g = line_string![
        (x: -1.0, y: 2.0), (x: 1.0, y:2.0),
        (x: 1.0, y: 2.0), (x: 5.0, y: 2.0)
    ];
    rect.clip(&g.into(), false);
}

fn polyclip_klippa() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = polygon![(x: 1.0, y: 1.0), (x: 5.0, y: 5.0)];
    rect.clip(&Geometry::Polygon(g)).unwrap();
}

fn polyclip_geo() {
    let rect = geo::Rect::new(coord! {x: 0., y: 0.}, coord! {x: 4., y: 4.}).to_polygon();
    let g = polygon![(x: 1.0, y: 1.0), (x: 5.0, y: 5.0)];
    rect.intersection(&g);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("lineclip_klippa", |b| b.iter(|| lineclip_klippa()));
    c.bench_function("lineclip_geo", |b| b.iter(|| lineclip_geo()));
    c.bench_function("linestringclip_klippa", |b| {
        b.iter(|| linestringclip_klippa())
    });
    c.bench_function("linestringclip_geo", |b| b.iter(|| linestringclip_geo()));
    c.bench_function("polyclip_klippa", |b| b.iter(|| polyclip_klippa()));
    c.bench_function("polyclip_geo", |b| b.iter(|| polyclip_geo()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
