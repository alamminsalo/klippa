use crate::geom::{CoordExt, LineExt};
use geo_types::{Coord, CoordFloat, Line, Point};

pub struct Rect<T: CoordFloat> {
    // bounding coordinates
    pub x0: T,
    pub y0: T,
    pub x1: T,
    pub y1: T,

    // rect lines
    pub lines: [Line<T>; 4],
}

impl<T: CoordFloat> Rect<T> {
    pub fn new(x0: T, y0: T, x1: T, y1: T) -> Self {
        // clockwise
        let lines = [
            Line::new((x0, y1), (x1, y1)),
            Line::new((x1, y1), (x1, y0)),
            Line::new((x1, y0), (x0, y0)),
            Line::new((x0, y0), (x0, y1)),
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

    // Returns true if perimeter index a is closer to i than b
    pub fn is_index_closer(&self, i: f64, mut a: f64, mut b: f64) -> bool {
        // println!("is_index_closer: i={i} -> a={a} b={b}");
        // wrap points around
        if a < i {
            a += 4.0;
        }
        if b < i {
            b += 4.0;
        }

        a < b
    }

    // Returns corner nodes between given perimeter indexes
    pub fn corner_nodes_between(&self, a: f64, mut b: f64) -> Vec<Coord<T>> {
        // wrap around if b point is before a
        if b < a {
            b += 4.0;
        }
        //println!("nodes between: {a}, {b}");

        // truncate to indexes
        let i = a.ceil() as usize;
        let j = b as usize;

        //println!("a={a}, b={b}, i={i}, j={j}");

        (i..=j)
            .map(|i| {
                let c = self.lines[i % 4].start;
                // println!("push {i} -> {c:?}");
                c
            })
            .collect()
    }
}
