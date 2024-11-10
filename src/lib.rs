mod geom;
mod rect;
mod util;

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

    fn clip_linestring(&self, g: &LineString<T>) -> MultiLineString<T> {
        self.inner
            .clip_segments(&g.lines().collect::<Vec<Line<T>>>())
            .into_iter()
            .map(util::segments_to_linestring)
            .collect()
    }

    // Clips and sews polygon ring back together by using corner points when necessary.
    fn clip_polygon_ring(&self, g: &LineString<T>) -> Vec<LineString<T>> {
        let linestrings: Vec<LineString<T>> = self
            .inner
            .clip_segments(&g.lines().collect::<Vec<Line<T>>>())
            .into_iter()
            .map(util::segments_to_linestring)
            .collect();

        // return on empty or single
        if linestrings.len() < 2 {
            return linestrings;
        }

        // add perimeter index of first coordinate
        let mut linestrings: Vec<(f64, LineString<T>)> = linestrings
            .into_iter()
            .map(|g| (self.inner.perimeter_index(&g[0]), g))
            .collect();

        // sort by perimeter index
        linestrings.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut joined = vec![];

        while !linestrings.is_empty() {
            // connect last point of A to first point of B
            let (a_idx, mut a) = linestrings.pop().unwrap();
            let (c_idx, mut c) = linestrings.pop().unwrap();

            // create a new segment passed from corner nodes
            let mut b = self.inner.corner_nodes_between(a_idx, c_idx);

            // join a-b-c
            a.0.extend(b.drain(..));
            a.0.extend(c.0.drain(..));

            joined.push(a);
        }

        joined
    }

    fn clip_polygon(&self, g: &Polygon<T>) -> Option<MultiPolygon<T>> {
        let exteriors = self.clip_polygon_ring(g.exterior());

        if exteriors.is_empty() {
            return None;
        }

        //let interiors = g
        //    .interiors()
        //    .into_iter()
        //    .filter_map(|ls| self.clip_polygon_ring(ls))
        //    .collect::<Vec<LineString<T>>>();

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

#[cfg(test)]
mod tests {
    use super::*;

    fn lines_to_poly<T: CoordFloat>(lines: &[Line<T>]) -> Polygon<T> {
        Polygon::new(util::segments_to_linestring(lines.to_vec()), vec![])
    }

    #[test]
    fn test_poly_simple() {
        let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
        let ls = util::segments_to_linestring(Rect::new(1.0, 1.0, 5.0, 5.0).lines.to_vec());

        let clip = rect.clip_polygon_ring(&ls);
        println!("{clip:?}");

        assert_eq!(clip.len(), 1);
    }
}
