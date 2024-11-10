use crate::geom::*;
use geo_types::Line;

#[test]
fn test_intersection() {
    //  |
    // -x-
    //  |
    let a = Line::new((0.0, -4.0), (0.0, 4.0));
    let b = Line::new((1.0, 0.0), (-1.0, 0.0));
    assert_eq!(a.intersection(&b), Some((0.0, 0.0).into()));
    assert_eq!(a.intersection(&b.reverse()), Some((0.0, 0.0).into()));

    //  |/
    //  x
    // /|
    let a = Line::new((0.0, 0.0), (0.0, 4.0));
    let b = Line::new((-1.0, 0.0), (1.0, 4.0));
    assert_eq!(a.intersection(&b), Some((0.0, 2.0).into()));
    assert_eq!(a.intersection(&b.reverse()), Some((0.0, 2.0).into()));

    //   /
    // -x---
    // /
    let a = Line::new((0.0, 0.0), (4.0, 0.0));
    let b = Line::new((4.0, 1.0), (0.0, -1.0));
    assert!(a.intersection(&b).is_some());
    assert!(a.intersection(&b.reverse()).is_some());

    //    |
    // ---x
    //
    let a = Line::new((0.0, 0.0), (4.0, 0.0));
    let b = Line::new((4.0, 4.0), (4.0, 0.0));
    assert!(!a.intersection(&b).is_some());
    assert!(!a.intersection(&b.reverse()).is_some());

    // Non-intersecting tests
    let a = Line::new((0.0, 0.0), (0.0, 4.0));
    let b = Line::new((1.0, 1.0), (0.1, 1.0));
    assert!(!a.intersection(&b).is_some());
    assert!(!a.intersection(&b.reverse()).is_some());

    let a = Line::new((0.0, 0.0), (0.0, 4.0));
    let b = Line::new((1.0, 1.0), (4.0, 4.0));
    assert!(!a.intersection(&b).is_some());
    assert!(!a.intersection(&b.reverse()).is_some());
}
