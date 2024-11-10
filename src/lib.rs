mod geom;

use geom::{Point, Segment};
use num_traits::Float;

pub struct Rect<T: Float> {
    x0: T,
    y0: T,
    x1: T,
    y1: T,
    sides: [Segment<T>; 4],
}

impl<T: Float + std::fmt::Debug> Rect<T> {
    pub fn new(x0: T, y0: T, x1: T, y1: T) -> Self {
        let sides = [
            Segment::new((x0, y0), (x1, y0)),
            Segment::new((x1, y0), (x1, y1)),
            Segment::new((x1, y1), (x0, y1)),
            Segment::new((x0, y1), (x0, y0)),
        ];
        Self {
            x0,
            y0,
            x1,
            y1,
            sides,
        }
    }

    fn contains_point(&self, p: &Point<T>) -> bool {
        self.x0 <= p.0 && p.0 <= self.x1 && self.y0 <= p.1 && p.1 <= self.y1
    }

    fn contains_segment(&self, s: &Segment<T>) -> bool {
        self.contains_point(&s.0) && self.contains_point(&s.1)
    }

    fn is_crossing(&self, s: &Segment<T>) -> bool {
        !self.contains_point(&s.0) && !self.contains_point(&s.1)
    }

    // Clips segment to this rectangle.
    pub fn clip_segment(&self, seg: &Segment<T>) -> Option<Segment<T>> {
        // Check if fully inside rect
        if self.contains_segment(&seg) {
            return Some(seg.clone());
        }

        // Find intersection points
        let mut isects = self
            .sides
            .iter()
            .filter_map(|side| side.isect(&seg))
            .take(2);

        let p1 = isects.next();
        let mut p2 = isects.next();

        // Line is crossing when both points are outside the rectangle.
        let is_crossing = self.is_crossing(seg);

        // Set p2 to None incase theyre the same point!
        // This fixes segments crossing corner point.
        if p1 == p2 {
            if is_crossing {
                // Segment is crossing rectangle by touching the corner point.
                // This would produce a segment with same point twice and does not qualify as
                // segment. Therefore let's return None.
                return None;
            } else {
                p2 = None;
            }
        }

        match (p1, p2) {
            // Two intersections:
            // Create new segment from intersection points.
            // To preserve direction, check which original point is closer to first point
            // and reverse or keep current direction accordingly.
            (Some(p1), Some(p2)) => {
                if seg.0.dist_manhattan(&p1) <= seg.0.dist_manhattan(&p2) {
                    Some(Segment(p1, p2))
                } else {
                    Some(Segment(p2, p1))
                }
            }

            // Single intersection:
            // Clip from edge to inside point.
            (Some(p1), None) => {
                if self.contains_point(&seg.0) {
                    Some(Segment(seg.0.clone(), p1))
                } else {
                    Some(Segment(p1, seg.1.clone()))
                }
            }

            // No intersections
            _ => None,
        }
    }

    // Clips segments to this rectangle.
    // Returns vector of grouped continuous segments.
    pub fn clip_segments(&self, segments: &[Segment<T>]) -> Vec<Vec<Segment<T>>> {
        // Get clipped segments
        let segments: Vec<Segment<T>> = segments
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
            if segments[i].1 != segments[(i + 1) % seg_len].0 {
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
            .fold(vec![], |mut acc: Vec<Vec<Segment<T>>>, seg: Segment<T>| {
                if let Some(segs) = acc.last_mut() {
                    if let Some(last) = segs.last() {
                        if last.1 == seg.0 {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_single() {
        let rect = Rect::new(0.0, 0.0, 4.0, 4.0);

        // should be contained fully
        assert!(rect
            .clip_segment(&Segment::new((0.0, 0.0), (1.0, 1.0)))
            .is_some());

        // should be contained fully
        assert!(rect
            .clip_segment(&Segment::new((0.0, 0.0), (4.0, 4.0)))
            .is_some());

        // should be clipped twice
        assert_eq!(
            rect.clip_segment(&Segment::new((-1.0, 1.0), (5.0, 1.0))),
            Some(Segment::new((0.0, 1.0), (4.0, 1.0)))
        );

        // should be clipped one time
        assert_eq!(
            rect.clip_segment(&Segment::new((1.0, 1.0), (1.0, 5.0))),
            Some(Segment::new((1.0, 1.0), (1.0, 4.0)))
        );

        // should be clipped one time
        assert_eq!(
            rect.clip_segment(&Segment::new((0.0, 0.0), (5.0, 5.0))),
            Some(Segment::new((0.0, 0.0), (4.0, 4.0)))
        );

        // should be clipped twice
        assert_eq!(
            rect.clip_segment(&Segment::new((-1.0, -1.0), (5.0, 5.0))),
            Some(Segment::new((0.0, 0.0), (4.0, 4.0)))
        );

        // should be left out
        assert!(rect
            .clip_segment(&Segment::new((5.0, 5.0), (6.0, 6.0)))
            .is_none());

        // corner-crossing case: should be left out
        assert_eq!(
            rect.clip_segment(&Segment::new((-1.0, 1.0), (1.0, -1.0))),
            None
        );

        // cross corner other time with ever so slight nudge
        assert!(rect
            .clip_segment(&Segment::new((-1.0, 1.0), (1.01, -1.0)))
            .is_some(),);
    }

    #[test]
    fn test_clip_multi() {
        let rect = Rect::new(0.0, 0.0, 4.0, 4.0);

        assert_eq!(
            rect.clip_segments(&vec![
                Segment::new((-1.0, 2.0), (1.0, 2.0)),
                Segment::new((1.0, 2.0), (5.0, 2.0)),
            ]),
            vec![vec![
                Segment::new((0.0, 2.0), (1.0, 2.0)),
                Segment::new((1.0, 2.0), (4.0, 2.0)),
            ]]
        );

        assert_eq!(
            rect.clip_segments(&vec![
                Segment::new((-1.0, 2.0), (1.0, 2.0)),
                Segment::new((1.0, 2.0), (5.0, 2.0)),
                Segment::new((5.0, 2.0), (7.0, 7.0)),
            ]),
            vec![vec![
                Segment::new((0.0, 2.0), (1.0, 2.0)),
                Segment::new((1.0, 2.0), (4.0, 2.0)),
            ]]
        );

        assert_eq!(
            rect.clip_segments(&vec![
                Segment::new((1.0, 2.0), (5.0, 2.0)),
                Segment::new((5.0, 2.0), (3.0, 4.0)),
            ]),
            vec![
                vec![Segment::new((4.0, 3.0), (3.0, 4.0)),],
                vec![Segment::new((1.0, 2.0), (4.0, 2.0)),],
            ]
        );

        assert_eq!(
            rect.clip_segments(&vec![
                Segment::new((2.0, 4.0), (4.0, 2.0)),
                Segment::new((4.0, 2.0), (2.0, 0.0))
            ])
            .len(),
            1
        );

        assert_eq!(
            rect.clip_segments(&vec![
                Segment::new((2.0, 4.0), (6.0, 2.0)),
                Segment::new((6.0, 2.0), (2.0, 0.0))
            ])
            .len(),
            2
        );

        // non-clipping segments
        assert!(rect
            .clip_segments(&vec![
                Segment::new((5.0, 2.0), (5.0, 4.0)),
                Segment::new((5.0, 4.0), (7.0, 0.0))
            ])
            .is_empty(),);
    }

    #[test]
    fn test_another_rect() {
        let rect = Rect::new(0.0, 0.0, 4.0, 4.0);

        // make another larger rectangle and tests against it's segments
        let segments = Rect::new(-1.0, -1.0, 5.0, 5.0).sides;
        assert!(rect.clip_segments(&segments).is_empty(),);

        // make small rect fully inside
        let segments = Rect::new(1.0, 1.0, 3.0, 3.0).sides;
        assert_eq!(rect.clip_segments(&segments), vec![segments.to_vec()]);

        // make small rect partially inside
        let segments = Rect::new(1.0, 5.0, 3.0, 1.0).sides;
        assert_eq!(
            rect.clip_segments(&segments),
            vec![vec![
                Segment::new((3.0, 4.0), (3.0, 1.0)),
                Segment::new((3.0, 1.0), (1.0, 1.0)),
                Segment::new((1.0, 1.0), (1.0, 4.0)),
            ]]
        );

        // another small rect partially inside
        let segments = Rect::new(1.0, 5.0, 5.0, 1.0).sides;
        assert_eq!(
            rect.clip_segments(&segments),
            vec![vec![
                Segment::new((4.0, 1.0), (1.0, 1.0)),
                Segment::new((1.0, 1.0), (1.0, 4.0)),
            ]]
        );

        // corner-crossing rectangle should produce no segments
        let segments = Rect::new(-1.0, 4.0, 0.0, 5.0).sides;
        assert!(rect.clip_segments(&segments).is_empty(),);
    }
}
