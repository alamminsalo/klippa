mod geom;
mod rect;

use geo_types::{CoordFloat, Geometry, Line, LineString, MultiLineString, MultiPolygon, Polygon};
use rect::Rect;

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

    fn segments_to_linestring(mut segments: Vec<Line<T>>) -> LineString<T> {
        // Take last segment aside, this doubles as empty checking
        if let Some(last) = segments.pop() {
            segments
                .into_iter()
                .map(|seg| seg.start)
                .chain([last.start, last.end])
                .collect()
        } else {
            LineString::new(vec![])
        }
    }

    fn clip_linestring(&self, g: &LineString<T>) -> MultiLineString<T> {
        self.inner
            .clip_segments(&g.lines().collect::<Vec<Line<T>>>())
            .into_iter()
            .map(Self::segments_to_linestring)
            .collect()
    }

    // Clips and sews polygon ring back together by using corner points when necessary.
    fn clip_polygon_ring(&self, g: &LineString<T>) -> Option<LineString<T>> {
        let mut groups: Vec<Vec<Line<T>>> = self
            .inner
            .clip_segments(&g.lines().collect::<Vec<Line<T>>>());

        // return on empty
        if groups.is_empty() {
            return None;
        }
        // return on single group
        if groups.len() == 1 {
            return Some(Self::segments_to_linestring(groups.pop().unwrap()));
        }

        // add perimeter index of first segment start point for each group
        let mut groups: Vec<(f64, Vec<Line<T>>)> = groups
            .into_iter()
            .map(|g| (self.inner.perimeter_index(&g[0].end), g))
            .collect();

        // sort by perimeter index
        groups.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        for i in 0..groups.len() {
            let b = {
                // connect last point of A to first point of B
                let (a_idx, a) = &groups[i];
                let (c_idx, c) = &groups[(i + 1) % groups.len()];

                // create a new segment, including rect corner nodes
                let mut b = vec![a.last().unwrap().end];
                b.extend(self.inner.corner_nodes_between(*a_idx, *c_idx));
                b.push(c.first().unwrap().start);

                geom::coords_to_lines(b)
            };

            // append b to a
            groups.get_mut(i).unwrap().1.extend(b);
        }

        // fold into single segment list
        let segments = groups
            .into_iter()
            .map(|(_, g)| g)
            .fold(vec![], |mut acc, g| {
                acc.extend(g);
                acc
            });

        Some(Self::segments_to_linestring(segments))
    }

    fn clip_polygon(&self, g: &Polygon<T>) -> Option<MultiPolygon<T>> {
        let exteriors = self.clip_polygon_ring(g.exterior());

        if exteriors.is_none() {
            return None;
        }

        let interiors = g
            .interiors()
            .into_iter()
            .filter_map(|ls| self.clip_polygon_ring(ls))
            .collect::<Vec<LineString<T>>>();

        todo!("place inner rings to exteriors")
    }

    pub fn clip(&self, g: &Geometry<T>) -> Option<Geometry<T>> {
        use Geometry::*;

        match g {
            Point(g) => self.inner.clip_point(g).and_then(|p| Some(Point(p))),
            Line(g) => self.inner.clip_segment(g).and_then(|l| Some(Line(l))),
            LineString(g) => Some(MultiLineString(self.clip_linestring(g))),
            Polygon(g) => self.clip_polygon(g).and_then(|p| Some(MultiPolygon(p))),
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
                    .filter_map(|poly| self.clip_polygon(poly))
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
