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
        let p2 = isects.next();

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
        let seg_len = segments.len();

        // Find beginning of first continuous string of segments
        // since we must assume first and last segments are connected.
        let mut i = 1;
        while i < seg_len {
            if segments[i - 1].1 != segments[i].0 {
                break;
            }
            i += 1
        }

        // Group segments, starting from i and looping around to beginning.
        let groups = segments.into_iter().cycle().skip(i).take(seg_len).fold(
            vec![],
            |mut acc: Vec<Vec<Segment<T>>>, seg: Segment<T>| {
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
            },
        );

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
        let seg = Segment::new((0.0, 0.0), (1.0, 1.0));
        assert!(rect.clip_segment(&seg).is_some());

        // should be clipped twice
        let seg = Segment::new((-1.0, 1.0), (5.0, 1.0));
        assert_eq!(
            rect.clip_segment(&seg),
            Some(Segment::new((0.0, 1.0), (4.0, 1.0)))
        );

        // should be clipped one time
        let seg = Segment::new((1.0, 1.0), (1.0, 5.0));
        assert_eq!(
            rect.clip_segment(&seg),
            Some(Segment::new((1.0, 1.0), (1.0, 4.0)))
        );

        // should be left out
        let seg = Segment::new((5.0, 5.0), (6.0, 6.0));
        assert!(rect.clip_segment(&seg).is_none());
    }
}
