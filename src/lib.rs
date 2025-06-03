pub mod geom;
pub mod rect;
mod util;

use geo_types::{CoordFloat, Geometry, Line, LineString, MultiLineString, MultiPolygon, Polygon};
use geom::{CoordExt, Reverse};
use log::debug;
pub use rect::Rect;

// Abstraction over crate::rect::Rect for handling complex geo types.
pub struct ClipRect<T: CoordFloat> {
    inner: Rect<T>,
}

impl<T: CoordFloat> ClipRect<T> {
    pub fn new(x0: T, y0: T, x1: T, y1: T) -> Self {
        Self {
            inner: Rect::new(x0, y0, x1, y1),
        }
    }

    fn clip_linestring(&self, g: &LineString<T>) -> MultiLineString<T> {
        self.inner
            .clip_segments(&g.lines().collect::<Vec<Line<T>>>())
            .into_iter()
            .map(util::segments_to_linestring)
            .collect()
    }

    // Clips and sews polygon ring back together by using corner points when necessary.
    fn clip_polygon_ring(&self, g: &LineString<T>) -> Vec<LineString<T>> {
        let input_lines = g.lines().collect::<Vec<Line<T>>>();

        let mut queue: Vec<(f64, LineString<T>)> = self
            .inner
            .clip_segments(&input_lines)
            .into_iter()
            .map(util::segments_to_linestring)
            .map(|g| (self.inner.perimeter_index(&g[0]), g))
            .collect();

        // When no intersections are found, check if clipping rectangle is fully contained by the
        // subject polygon. In that case, bounds of the clipping rectangle.
        if queue.is_empty() && self.inner.is_contained(&input_lines) {
            debug!("clipping rect inside geom");
            return vec![util::segments_to_linestring(self.inner.lines.to_vec())];
        }

        // sort elements with starting point perimeter index
        queue.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // begin connect loop
        let mut output = vec![];

        while !queue.is_empty() {
            debug!("step");
            util::print_queue(&queue);

            // pop last element of the vector, containing the smallest perimeter index
            let (p_a, mut a) = queue.pop().unwrap();

            if a.is_closed() {
                debug!("push");
                output.push(a);
                continue;
            }

            // Check if head point of a is closer than next in queue
            let p_tail = self.inner.perimeter_index(&a.0.last().unwrap());
            debug!("p_tail={p_tail}");

            // Find next value with greater perimeter index than the p_tail
            if let Some(next) = queue
                .iter()
                .enumerate()
                .rev()
                .find(|(_, (p_idx, _))| self.inner.is_index_closer(p_tail, *p_idx, p_a))
                .map(|(idx, _)| idx)
            {
                let (p_b, b) = queue.remove(next);
                debug!("join lines {p_b}, {b:?}");
                // create a new segment passed from corner nodes
                let corners = self.inner.corner_nodes_between(p_tail, p_b);

                // connect last point of C to first point of A
                debug!("connect: {a:?} -> {corners:?} -> {b:?}");

                // join C-B-A and push back into queue
                a.0.extend(corners);
                a.0.extend(b);

                queue.push((p_a, a));
            } else {
                // Close line with self
                debug!("close line {p_a} -> {p_tail}");

                let corners = self.inner.corner_nodes_between(p_tail, p_a);
                a.0.extend(corners);
                a.0.push(a[0].clone());

                queue.push((p_a, a));
            }
        }

        debug!("out");
        output
    }

    fn clip_polygon(&self, g: &Polygon<T>) -> MultiPolygon<T> {
        let mut polys: Vec<Polygon<T>> = self
            .clip_polygon_ring(g.exterior())
            .into_iter()
            .filter_map(|ls| {
                if ls.points().len() >= 3 {
                    Some(Polygon::new(ls, vec![]))
                } else {
                    None
                }
            })
            .collect();

        // clip and place interiors to polys
        if !polys.is_empty() {
            g.interiors()
                .into_iter()
                .map(|ls| self.clip_polygon_ring(&ls.clone().reverse()))
                .flatten()
                .filter_map(|ls| {
                    if ls.points().len() >= 3 {
                        Some(ls)
                    } else {
                        None
                    }
                })
                .for_each(|hole| {
                    if polys.len() == 1 {
                        // single poly -> no need to find
                        polys[0].interiors_push(hole.reverse());
                    } else {
                        // find parent poly
                        for poly in polys.iter_mut() {
                            if let Some(c) = util::find_coord_inside(&hole, &self.inner) {
                                debug!("coord inside");
                                if c.is_inside(poly.exterior()) {
                                    debug!("is inside");
                                    poly.interiors_push(hole.reverse());
                                    break;
                                }
                            }
                        }
                    }
                });
        }

        polys.into()
    }

    pub fn clip(&self, g: &Geometry<T>) -> Option<Geometry<T>> {
        use Geometry::*;

        match g {
            Point(g) => self.inner.clip_point(g).and_then(|p| Some(Point(p))),
            Line(g) => self.inner.clip_segment(g).and_then(|l| Some(Line(l))),
            LineString(g) => {
                let g = self.clip_linestring(g);
                if g.0.is_empty() {
                    None
                } else if g.0.len() == 1 {
                    Some(LineString(g.into_iter().next().unwrap()))
                } else {
                    Some(MultiLineString(g))
                }
            }
            Polygon(g) => {
                let g = self.clip_polygon(g);
                if g.0.is_empty() {
                    None
                } else if g.0.len() == 1 {
                    Some(Polygon(g.into_iter().next().unwrap()))
                } else {
                    Some(MultiPolygon(g))
                }
            }
            MultiPoint(g) => Some(MultiPoint(
                g.into_iter()
                    .filter_map(|p| self.inner.clip_point(p))
                    .collect(),
            )),
            MultiLineString(g) => Some(MultiLineString(
                g.into_iter()
                    .map(|ls| self.clip_linestring(ls))
                    .flatten()
                    .collect(),
            )),
            MultiPolygon(g) => {
                let polys: Vec<_> = g
                    .into_iter()
                    .map(|poly| self.clip_polygon(poly))
                    .flatten()
                    .collect();

                if polys.is_empty() {
                    None
                } else {
                    Some(MultiPolygon(polys.into()))
                }
            }
            _ => None,
        }
    }
}
