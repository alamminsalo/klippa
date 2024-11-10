use crate::geom::{CoordExt, LineExt};
use geo_types::{Coord, CoordFloat, Line, Point};

pub(crate) struct Rect<T: CoordFloat> {
    // bounding coordinates
    x0: T,
    y0: T,
    x1: T,
    y1: T,

    // rect lines
    lines: [Line<T>; 4],
}

impl<T: CoordFloat> Rect<T> {
    pub fn new(x0: T, y0: T, x1: T, y1: T) -> Self {
        let lines = [
            Line::new((x0, y0), (x1, y0)),
            Line::new((x1, y0), (x1, y1)),
            Line::new((x1, y1), (x0, y1)),
            Line::new((x0, y1), (x0, y0)),
        ];

        Self {
            x0,
            y0,
            x1,
            y1,
            lines,
        }
    }

    fn corner_points(&self) -> [&Coord<T>; 4] {
        [
            &self.lines[0].start,
            &self.lines[1].start,
            &self.lines[2].start,
            &self.lines[3].start,
        ]
    }

    fn contains_coord(&self, c: &Coord<T>) -> bool {
        self.x0 <= c.x && c.x <= self.x1 && self.y0 <= c.y && c.y <= self.y1
    }

    fn contains_segment(&self, s: &Line<T>) -> bool {
        self.contains_coord(&s.start) && self.contains_coord(&s.end)
    }

    // Line is crossing when both points are outside the rectangle.
    fn is_crossing(&self, s: &Line<T>) -> bool {
        !self.contains_coord(&s.start) && !self.contains_coord(&s.end)
    }

    fn is_corner(&self, p: &Coord<T>) -> bool {
        self.corner_points().into_iter().any(|corner| p == corner)
    }

    pub fn clip_point(&self, p: &Point<T>) -> Option<Point<T>> {
        if self.contains_coord(&p.0) {
            Some(*p)
        } else {
            None
        }
    }

    pub fn clip_segment(&self, seg: &Line<T>) -> Option<Line<T>> {
        // Check if fully inside rect
        if self.contains_segment(&seg) {
            return Some(seg.clone());
        }

        // Find unique intersection points
        let mut isects = self
            .lines
            .iter()
            .filter_map(|side| side.intersection(seg))
            .fold(vec![], |mut acc, p| {
                if !acc.contains(&p) {
                    acc.push(p);
                }
                acc
            })
            .into_iter();

        let p1 = isects.next();
        let p2 = isects.next();

        match (p1, p2) {
            // Two intersections:
            // Create new segment from intersection points.
            // To preserve direction, check which original point is closer to first point
            // and reverse or keep current direction accordingly.
            (Some(p1), Some(p2)) => {
                if seg.start.manhattan_dist(&p1) <= seg.start.manhattan_dist(&p2) {
                    Some(Line::new(p1, p2))
                } else {
                    Some(Line::new(p2, p1))
                }
            }

            // Single intersection:
            // Clip from edge to inside point.
            (Some(p1), None) => {
                if self.is_crossing(seg) && self.is_corner(&p1) {
                    // Line is crossing rectangle by touching the corner point.
                    // This would produce a segment with same point twice and does not qualify as
                    // segment. Therefore let's return None.
                    return None;
                }

                // Decide segment direction
                if self.contains_coord(&seg.start) {
                    Some(Line::new(seg.start.clone(), p1))
                } else {
                    Some(Line::new(p1, seg.end.clone()))
                }
            }

            // No intersections
            _ => None,
        }
    }

    // Returns vector of grouped continuous segments.
    pub fn clip_segments(&self, segments: &[Line<T>]) -> Vec<Vec<Line<T>>> {
        // Get clipped segments
        let segments: Vec<Line<T>> = segments
            .into_iter()
            .filter_map(|seg| self.clip_segment(seg))
            .collect();

        // Early return on empty segments list
        if segments.is_empty() {
            return vec![];
        }

        // Find first splitpoint
        // since we must assume first and last segments are connected.
        let seg_len = segments.len();
        let mut offset = 0;
        for i in 0..segments.len() {
            if segments[i].end != segments[(i + 1) % seg_len].start {
                offset = (i + 1) % seg_len;
                break;
            }
        }

        // Group segments, starting from offset and looping around
        let groups = segments
            .into_iter()
            .cycle()
            .skip(offset)
            .take(seg_len)
            .fold(vec![], |mut acc: Vec<Vec<Line<T>>>, seg: Line<T>| {
                if let Some(segs) = acc.last_mut() {
                    if let Some(last) = segs.last() {
                        if last.end == seg.start {
                            // Continue segment group
                            segs.push(seg);
                        } else {
                            // Start another group
                            acc.push(vec![seg]);
                        }
                    }
                } else {
                    acc.push(vec![seg]);
                }

                acc
            });

        groups
    }

    // Indexes a point along the rect perimeter in 0..4
    // Can be used to sort intersection points.
    pub fn perimeter_index(&self, p: &Coord<T>) -> f64 {
        let mut f: f64 = 0.0;
        let corners = self.corner_points();

        for i in 0..4 {
            let c1 = corners[i];
            let c2 = corners[(i + 1) % 4];

            if i % 2 == 0 {
                if p.y == c1.y {
                    f += (p.x - c1.x).to_f64().unwrap() / (c2.x - c1.x).to_f64().unwrap();
                    break;
                }
            } else {
                if p.x == c1.x {
                    f += (p.y - c1.y).to_f64().unwrap() / (c2.y - c1.y).to_f64().unwrap();
                    break;
                }
            }

            f += 1.0;
        }

        f
    }

    // Returns corner nodes between given perimeter indexes
    pub fn corner_nodes_between(&self, a: f64, mut b: f64) -> Vec<Coord<T>> {
        // wrap around if b point is before a
        if b < a {
            b += 4.0;
        }

        // truncate to indexes
        let i = a as usize;
        let j = b as usize;

        let mut res = Vec::with_capacity(4);

        for i in i..j {
            res.push(self.lines[i % 4].end);
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use geo_types::coord;

    use super::*;

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

        // make small rect partially inside
        let segments = Rect::new(1.0, 5.0, 3.0, 1.0).lines;
        assert_eq!(
            rect.clip_segments(&segments),
            vec![vec![
                Line::new((3.0, 4.0), (3.0, 1.0)),
                Line::new((3.0, 1.0), (1.0, 1.0)),
                Line::new((1.0, 1.0), (1.0, 4.0)),
            ]]
        );

        // another small rect partially inside
        let segments = Rect::new(1.0, 5.0, 5.0, 1.0).lines;
        assert_eq!(
            rect.clip_segments(&segments),
            vec![vec![
                Line::new((4.0, 1.0), (1.0, 1.0)),
                Line::new((1.0, 1.0), (1.0, 4.0)),
            ]]
        );

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
        assert_eq!(rect.perimeter_index(&coord! {x: 4.0, y: 4.0}), 2.0,);
        assert_eq!(rect.perimeter_index(&coord! {x: 2.0, y: 4.0}), 2.5);
        assert_eq!(rect.perimeter_index(&coord! {x: 0.0, y: 4.0}), 3.0,);
        assert_eq!(rect.perimeter_index(&coord! {x: 0.0, y: 1.0}), 3.75,);

        // test finding corner nodes between indexes
        assert_eq!(rect.corner_nodes_between(0.1, 1.1).len(), 1);
        assert_eq!(rect.corner_nodes_between(1.1, 0.1).len(), 3);
        assert_eq!(rect.corner_nodes_between(3.9, 0.1).len(), 1);
    }
}
