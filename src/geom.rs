use crate::rect::Rect;
use geo_types::{Coord, CoordFloat, Line, LineString, Polygon};
use log::debug;

// Coord extension trait
pub trait CoordExt<T: CoordFloat> {
    fn yx(self) -> Self;
    fn manhattan_dist(&self, other: &Self) -> T;
    fn is_inside(&self, ls: &LineString<T>) -> bool;
}

impl<T: CoordFloat> CoordExt<T> for Coord<T> {
    fn yx(self) -> Self {
        (self.y, self.x).into()
    }

    fn manhattan_dist(&self, other: &Self) -> T {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    fn is_inside(&self, ls: &LineString<T>) -> bool {
        let ix = Line::new((self.x, self.y), (self.x, T::infinity()));
        ls.lines().filter_map(|l| ix.intersection(&l)).count() % 2 == 1
    }
}

// Line extension trait
pub trait LineExt<T: CoordFloat> {
    fn intersection(&self, other: &Line<T>) -> Option<Coord<T>>;
    fn is_vertical(&self) -> bool;
    fn is_ortho(&self) -> bool;
    fn swap_axes(self) -> Self;
}

impl<T: CoordFloat> LineExt<T> for Line<T> {
    // Checks if line B intersects A, where A (self) is axis-aligned line.
    // If the two lines share a same point, the result is None since clipping is not needed.
    fn intersection(&self, b: &Self) -> Option<Coord<T>> {
        let a = self;

        debug!("isect: {a:?} -> {b:?}");

        if !a.is_ortho() {
            panic!("non-orthogonal A");
        }

        // If A is not vertical line, invert axes
        if !a.is_vertical() {
            debug!("invert");
            return a
                .swap_axes()
                .intersection(&b.swap_axes())
                .and_then(|p| Some(p.yx()));
        }

        // Get X-axis differences
        let c = Line::new(b.start, a.start);
        let dx_c = c.dx();
        let dx_b = b.dx();

        debug!("dx_c={dx_c:?}, dx_b={dx_b:?}");

        // Check delta signatures and distances
        if dx_c.is_sign_positive() != dx_b.is_sign_positive() || dx_b.abs() <= dx_c.abs() {
            return None;
        }

        let d = Line::new(b.start, a.end);

        // Check if B is directed toward A:
        // Slope B must be in between of slopes C, D
        let slope_b = b.slope();
        let slope_c = c.slope();
        let slope_d = d.slope();

        debug!("slope_b={b:?}, slope_c={c:?}, slope_d={slope_d:?}");
        if slope_b < slope_c.min(slope_d) || slope_b > slope_c.max(slope_d) {
            return None;
        }

        Some(b.start + (dx_c, dx_c * slope_b).into())
    }

    fn is_vertical(&self) -> bool {
        self.start.x == self.end.x
    }

    fn is_ortho(&self) -> bool {
        self.start.x == self.end.x || self.start.y == self.end.y
    }

    fn swap_axes(self) -> Self {
        Self::new(self.start.yx(), self.end.yx())
    }
}

pub trait Reverse<T: CoordFloat> {
    fn reverse(self) -> Self;
}

impl<T: CoordFloat> Reverse<T> for Line<T> {
    fn reverse(self) -> Self {
        Self::new(self.end, self.start)
    }
}

impl<T: CoordFloat> Reverse<T> for LineString<T> {
    fn reverse(self) -> Self {
        Self::new(self.0.into_iter().rev().collect())
    }
}

pub trait PolygonExt<T: CoordFloat> {
    fn put_hole(&mut self, ls: LineString<T>, rect: &Rect<T>);
}

impl<T: CoordFloat> PolygonExt<T> for Polygon<T> {
    fn put_hole(&mut self, ls: LineString<T>, rect: &Rect<T>) {
        if ls.is_closed() {
            debug!("closed ring");
            self.interiors_push(ls);
        } else {
            // assume hole is cut
            self.exterior_mut(|ext| {
                let start = ls.0[0];
                let end = ls.0[ls.0.len() - 1];
                let num_corners = rect
                    .corner_nodes_between(rect.perimeter_index(&start), rect.perimeter_index(&end))
                    .len();

                debug!("{start:?} -> {end:?} corners={num_corners}");

                for i in 0..ext.0.len() - 1 {
                    let line = Line::new(ext.0[i], ext.0[i + 1]);
                    if line.is_ortho() && start.x == line.start.x || start.y == line.start.y {
                        debug!("rect line={line:?}");
                        // place linestring between coordinates
                        let (l, r) = ext.0.split_at(i + 1);
                        ext.0 = l
                            .iter()
                            .chain(&ls.0)
                            .chain(&r[num_corners..])
                            .copied()
                            .collect();

                        break;
                    }
                }
            });
        }
    }
}
