pub mod geom;
pub mod rect;
pub mod util;

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
        let mut queue: Vec<(f64, LineString<T>)> = self
            .inner
            .clip_segments(&g.lines().collect::<Vec<Line<T>>>())
            .into_iter()
            .map(util::segments_to_linestring)
            .map(|g| (self.inner.perimeter_index(&g[0]), g))
            .collect();

        // sort elements with starting point perimeter index
        queue.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        println!("{queue:?}");

        let mut output = vec![];

        // connect loop
        while !queue.is_empty() {
            println!("pop");
            let (a_idx, mut a) = queue.pop().unwrap();
            println!("a={a:?}");

            if a.is_closed() {
                output.push(a);
                continue;
            }

            // Check if head point of a is closer than next in queue
            let tail_idx = self.inner.perimeter_index(&a.0.last().unwrap());
            println!("tail_idx={tail_idx}");

            // Process ring
            if queue.is_empty()
                || self
                    .inner
                    .is_index_closer(tail_idx, a_idx, queue.last().unwrap().0)
            {
                // Close line
                println!("close line {a_idx} -> {tail_idx}");

                let corners = self.inner.corner_nodes_between(tail_idx, a_idx);
                a.0.extend(corners);
                a.0.push(a[0].clone());

                queue.push((a_idx, a));
            } else if let Some((b_idx, b)) = queue.pop() {
                println!("join {b_idx}, {b:?}");
                // create a new segment passed from corner nodes
                let corners = self.inner.corner_nodes_between(tail_idx, b_idx);

                // connect last point of C to first point of A
                println!("connect: {a:?} -> {corners:?} -> {b:?}");

                // join C-B-A and push back into queue
                a.0.extend(corners);
                a.0.extend(b);

                queue.push((a_idx, a));
            }
        }

        output
    }

    fn clip_polygon(&self, g: &Polygon<T>) -> MultiPolygon<T> {
        let exteriors = self.clip_polygon_ring(g.exterior());

        //let interiors = g
        //    .interiors()
        //    .into_iter()
        //    .filter_map(|ls| self.clip_polygon_ring(ls))
        //    .collect::<Vec<LineString<T>>>();

        // TODO: place inner rings to exteriors
        exteriors
            .into_iter()
            .map(|ls| Polygon::new(ls, vec![]))
            .collect()
    }

    pub fn clip(&self, g: &Geometry<T>) -> Option<Geometry<T>> {
        use Geometry::*;

        match g {
            Point(g) => self.inner.clip_point(g).and_then(|p| Some(Point(p))),
            Line(g) => self.inner.clip_segment(g).and_then(|l| Some(Line(l))),
            LineString(g) => Some(MultiLineString(self.clip_linestring(g))),
            Polygon(g) => {
                let g = self.clip_polygon(g);
                if g.0.is_empty() {
                    None
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

#[cfg(test)]
mod tests {
    use super::*;
    use geo::wkt;
    use geo::BoundingRect;
    use wkt::ToWkt;

    fn lines_to_poly<T: CoordFloat>(lines: &[Line<T>]) -> Polygon<T> {
        Polygon::new(util::segments_to_linestring(lines.to_vec()), vec![])
    }

    #[test]
    fn test_poly_diagonal() {
        let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
        let g = wkt!(POLYGON((0.2526855468749994 4.937724274302482,5.174560546875 0.0549316322096729,3.3508300781249996 -1.0436434559084802,-1.3073730468750009 4.039617826768435,0.2526855468749994 4.937724274302482)));

        let clip = rect.clip(&Geometry::Polygon(g)).unwrap();
        assert_eq!(
            clip.to_wkt().to_string(),
            "MULTIPOLYGON(((1.1979154595268593 4,4 1.2201655310741821,4 0,2.3944552358035858 0,0 2.612947796960526,0 4,1.1979154595268593 4)))"
        );
    }

    #[test]
    fn test_poly_angle() {
        let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
        let g = wkt!(POLYGON((2.7465820312500004 4.423090477960912,2.7026367187499996 3.19536379832941,4.746093749999999 3.217302058187144,4.7900390625 1.5159363834516881,1.109619140625 1.603794430058997,1.1755371093750002 4.543570279371764,2.7465820312500004 4.423090477960912)));

        let clip = rect.clip(&Geometry::Polygon(g)).unwrap();
        println!("{}", clip.to_wkt());
        assert_eq!(
            clip.to_wkt().to_string(),
            "MULTIPOLYGON(((2.731437908719937 4,2.7026367187499996 3.19536379832941,4 3.209292103333472,4 1.5347959946504444,1.109619140625 1.603794430058997,1.1633487485863934 4,2.731437908719937 4)))"
        );
    }

    #[test]
    fn test_poly_cross() {
        let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
        let g = wkt!(POLYGON((1.3732910156250002 4.532618393971788,2.867431640625 4.5764249358536375,2.933349609374999 2.8223442468940902,4.812011718749999 2.8113711933311407,4.822998046874999 1.537901237431484,3.021240234375 1.5488835798473986,3.0322265624999996 -0.3515602939922644,1.417236328125 -0.37353251022881295,1.3952636718749996 1.4939713066293194,-0.7690429687499999 1.482988685660274,-0.7360839843749998 2.8333171968552904,1.3293457031250002 2.7126091154394203,1.109619140625 4.4449973697272895,1.3732910156250002 4.532618393971788)));

        let clip = rect.clip(&Geometry::Polygon(g)).unwrap().to_wkt();
        println!("{clip}");
    }

    //#[test]
    //fn test_poly_star() {
    //    let bbox = wkt!(POLYGON((7.407106969698674 43.75241501165641,7.407106969698674 43.723294553130074,7.442523770497758 43.723294553130074,7.442523770497758 43.75241501165641,7.407106969698674 43.75241501165641))).bounding_rect().unwrap();
    //    let rect = ClipRect::new(bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y);
    //    let g = wkt!(POLYGON((7.4501895904541025 43.74815713922604,7.434310913085938 43.74648303737507,7.4274444580078125 43.75671292857024,7.417831420898437 43.746421032703665,7.399291992187499 43.74691706827676,7.4180030822753915 43.734142842434494,7.400150299072266 43.72024932899603,7.427873611450195 43.72626611210518,7.435512542724609 43.71907071307564,7.4362850189208975 43.73662348786766,7.4501895904541025 43.74815713922604)));

    //    let clip = rect.clip(&Geometry::Polygon(g)).unwrap().to_wkt();
    //    println!("{clip}");
    //}
    //#[test]
    //fn test_poly_concave() {
    //    let rect = ClipRect::new(0., 0., 4., 4.);
    //    let g = wkt!(POLYGON((5.053710937499999 2.339437582871412,3.1091308593749996 4.5764249358536375,1.7907714843749991 4.642129714308481,4.504394531249999 1.6257583604127603,0.5053710937499998 -0.46142079353060694,4.361572265624999 -0.8239462091017487,5.504150390624999 2.054003264372156,5.053710937499999 2.339437582871412)));

    //    let clip = rect.clip(&Geometry::Polygon(g)).unwrap().to_wkt();
    //    println!("{clip}");
    //}

    //#[test]
    //fn test_poly_complex() {
    //    let bbox = wkt!(POLYGON((7.407106969698674 43.75241501165641,7.407106969698674 43.723294553130074,7.442523770497758 43.723294553130074,7.442523770497758 43.75241501165641,7.407106969698674 43.75241501165641))).bounding_rect().unwrap();
    //    let rect = ClipRect::new(bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y);
    //    let g = wkt!(POLYGON((7.451820373535156 43.74803313328718,7.448129653930664 43.73544519409995,7.420148849487305 43.72161401320187,7.407960891723633 43.720807612655534,7.444267272949218 43.742452600140325,7.399034500122071 43.73947610307701,7.400150299072266 43.7479711302214,7.425727844238281 43.74685506405493,7.4221229553222665 43.75534904410739,7.42924690246582 43.74790912709145,7.446842193603515 43.74803313328718,7.446413040161132 43.75541103953009,7.437829971313477 43.75528704862043,7.438516616821289 43.751009204912606,7.4335384368896475 43.75045120275473,7.433023452758789 43.7571468852893,7.453279495239258 43.75720887884938,7.451820373535156 43.74803313328718)));

    //    let clip = rect.clip(&Geometry::Polygon(g)).unwrap().to_wkt();
    //    println!("{clip}");
    //}
}
