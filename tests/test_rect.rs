use geo_types::{coord, Line};
use klippa::*;

#[test]
fn test_clip_single() {
    let rect = Rect::new(0.0, 0.0, 4.0, 4.0);

    // should be contained fully
    assert!(rect
        .clip_segment(&Line::new((0.0, 0.0), (1.0, 1.0)))
        .is_some());

    // should be contained fully
    assert!(rect
        .clip_segment(&Line::new((0.0, 0.0), (4.0, 4.0)))
        .is_some());

    // should be clipped twice
    assert_eq!(
        rect.clip_segment(&Line::new((-1.0, 1.0), (5.0, 1.0))),
        Some(Line::new((0.0, 1.0), (4.0, 1.0)))
    );

    // should be clipped one time
    assert_eq!(
        rect.clip_segment(&Line::new((1.0, 1.0), (1.0, 5.0))),
        Some(Line::new((1.0, 1.0), (1.0, 4.0)))
    );

    // should be clipped one time
    assert_eq!(
        rect.clip_segment(&Line::new((0.0, 0.0), (5.0, 5.0))),
        Some(Line::new((0.0, 0.0), (4.0, 4.0)))
    );

    // should be clipped twice
    assert_eq!(
        rect.clip_segment(&Line::new((-1.0, -1.0), (5.0, 5.0))),
        Some(Line::new((0.0, 0.0), (4.0, 4.0)))
    );

    // should be left out
    assert!(rect
        .clip_segment(&Line::new((5.0, 5.0), (6.0, 6.0)))
        .is_none());

    // corner-crossing case: should be left out
    assert_eq!(
        rect.clip_segment(&Line::new((-1.0, 1.0), (1.0, -1.0))),
        None
    );

    // cross corner other time with ever so slight nudge
    assert!(rect
        .clip_segment(&Line::new((-1.0, 1.0), (1.01, -1.0)))
        .is_some(),);
}

#[test]
fn test_clip_multi() {
    let rect = Rect::new(0.0, 0.0, 4.0, 4.0);

    assert_eq!(
        rect.clip_segments(&vec![
            Line::new((-1.0, 2.0), (1.0, 2.0)),
            Line::new((1.0, 2.0), (5.0, 2.0)),
        ]),
        vec![vec![
            Line::new((0.0, 2.0), (1.0, 2.0)),
            Line::new((1.0, 2.0), (4.0, 2.0)),
        ]]
    );

    assert_eq!(
        rect.clip_segments(&vec![
            Line::new((-1.0, 2.0), (1.0, 2.0)),
            Line::new((1.0, 2.0), (5.0, 2.0)),
            Line::new((5.0, 2.0), (7.0, 7.0)),
        ]),
        vec![vec![
            Line::new((0.0, 2.0), (1.0, 2.0)),
            Line::new((1.0, 2.0), (4.0, 2.0)),
        ]]
    );

    assert_eq!(
        rect.clip_segments(&vec![
            Line::new((1.0, 2.0), (5.0, 2.0)),
            Line::new((5.0, 2.0), (3.0, 4.0)),
        ]),
        vec![
            vec![Line::new((4.0, 3.0), (3.0, 4.0)),],
            vec![Line::new((1.0, 2.0), (4.0, 2.0)),],
        ]
    );

    assert_eq!(
        rect.clip_segments(&vec![
            Line::new((2.0, 4.0), (4.0, 2.0)),
            Line::new((4.0, 2.0), (2.0, 0.0))
        ])
        .len(),
        1
    );

    assert_eq!(
        rect.clip_segments(&vec![
            Line::new((2.0, 4.0), (6.0, 2.0)),
            Line::new((6.0, 2.0), (2.0, 0.0))
        ])
        .len(),
        2
    );

    // non-clipping segments
    assert!(rect
        .clip_segments(&vec![
            Line::new((5.0, 2.0), (5.0, 4.0)),
            Line::new((5.0, 4.0), (7.0, 0.0))
        ])
        .is_empty(),);
}

#[test]
fn test_another_rect() {
    let rect = Rect::new(0.0, 0.0, 4.0, 4.0);

    // make another larger rectangle and tests against it's segments
    let segments = Rect::new(-1.0, -1.0, 5.0, 5.0).lines;
    assert!(rect.clip_segments(&segments).is_empty(),);

    // make small rect fully inside
    let segments = Rect::new(1.0, 1.0, 3.0, 3.0).lines;
    assert_eq!(rect.clip_segments(&segments), vec![segments.to_vec()]);

    // corner-crossing rectangle should produce no segments
    let segments = Rect::new(-1.0, 4.0, 0.0, 5.0).lines;
    assert!(rect.clip_segments(&segments).is_empty(),);
}

#[test]
fn test_self_crossing_segments() {
    let rect = Rect::new(0.0, 0.0, 4.0, 4.0);

    assert_eq!(
        rect.clip_segments(&vec![
            Line::new((-1.0, -1.0), (5.0, 5.0)),
            Line::new((5.0, 5.0), (5.0, -1.0)),
            Line::new((5.0, -1.0), (-1.0, 5.0)),
        ]),
        vec![
            vec![Line::new((4.0, 0.0), (0.0, 4.0))],
            vec![Line::new((0.0, 0.0), (4.0, 4.0))],
        ]
    );
}

#[test]
fn test_perimeter_index() {
    let rect = Rect::new(0.0, 0.0, 4.0, 4.0);

    // test perimeter index for coordinates
    assert_eq!(rect.perimeter_index(&coord! {x: 0.0, y: 0.0}), 0.0,);
    assert_eq!(rect.perimeter_index(&coord! {x: 3.0, y: 0.0}), 0.75);
    assert_eq!(rect.perimeter_index(&coord! {x: 4.0, y: 0.0}), 1.0,);
    assert_eq!(rect.perimeter_index(&coord! {x: 2.0, y: 0.0}), 0.5);
    assert_eq!(rect.perimeter_index(&coord! {x: 0.0, y: 4.0}), 3.0,);
    assert_eq!(rect.perimeter_index(&coord! {x: 0.0, y: 1.0}), 3.75,);

    // test finding corner nodes between indexes
    assert_eq!(rect.corner_nodes_between(0.1, 1.1).len(), 1);
    assert_eq!(rect.corner_nodes_between(1.1, 0.1).len(), 3);
    assert_eq!(rect.corner_nodes_between(3.9, 0.1).len(), 1);
}
