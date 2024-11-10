use std::fmt::Display;

use geom::{Point, Segment};
use num_traits::Float;

mod geom;

pub struct Rect<T: Float>(T, T, T, T);

impl<T: Float + Display> Rect<T> {
    fn contains_point(&self, p: &Point<T>) -> bool {
        self.0 >= p.0 && p.0 <= self.2 && self.1 >= p.1 && p.1 <= self.3
    }

    fn contains_segment(&self, s: &Segment<T>) -> bool {
        self.contains_point(&s.0) && self.contains_point(&s.1)
    }

    fn segments(&self) -> [Segment<T>; 4] {
        [
            Segment::new((self.0, self.0), (self.1, self.0)),
            Segment::new((self.1, self.0), (self.1, self.1)),
            Segment::new((self.1, self.1), (self.0, self.1)),
            Segment::new((self.0, self.1), (self.0, self.0)),
        ]
    }

    fn clip_segment(&self, seg: &Segment<T>) -> Option<Segment<T>> {
        // Check if fully inside rect
        if self.contains_segment(&seg) {
            return Some(seg.clone());
        }

        // Find intersection points
        let mut isects = self
            .segments()
            .into_iter()
            .filter_map(|side| side.isect(&seg))
            .take(2);

        // Check intersections
        if let Some(p1) = isects.next() {
            if let Some(p2) = isects.next() {
                // Two intersections:
                // Create new segment from intersection points.
                // TODO: preserve direction
                Some(Segment(p1, p2))
            } else {
                // Single intersection:
                // Clip from edge to inside point.
                if self.contains_point(&seg.0) {
                    Some(Segment(seg.0.clone(), p1))
                } else {
                    Some(Segment(p1, seg.1.clone()))
                }
            }
        } else {
            None
        }
    }

    fn clip(&self, segments: &[Segment<T>]) -> Vec<Segment<T>> {
        let sides = self.segments();

        for seg in segments {
            let isects = sides.iter().filter_map(|side| side.isect(seg)).take(2);
        }

        todo!()
    }
}
