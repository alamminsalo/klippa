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

    fn clip_polygon(&self, g: &Polygon<T>) -> MultiPolygon<T> {
        let exteriors = self.clip_linestring(g.exterior());
        let interiors = g
            .interiors()
            .into_iter()
            .map(|ls| self.clip_linestring(ls))
            .collect::<Vec<MultiLineString<T>>>();

        todo!("sew lines and merge into polygon")
    }

    pub fn clip(&self, g: &Geometry<T>) -> Option<Geometry<T>> {
        use Geometry::*;

        match g {
            Point(g) => self.inner.clip_point(g).and_then(|p| Some(Point(p))),
            Line(g) => self.inner.clip_segment(g).and_then(|l| Some(Line(l))),
            LineString(g) => Some(MultiLineString(self.clip_linestring(g))),
            Polygon(g) => Some(MultiPolygon(self.clip_polygon(g))),
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
            MultiPolygon(g) => Some(MultiPolygon(
                g.into_iter()
                    .map(|poly| self.clip_polygon(poly))
                    .flatten()
                    .collect(),
            )),
            _ => None,
        }
    }
}
