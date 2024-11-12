use geo_types::{Coord, CoordFloat, Line, LineString};

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
        let isect = Line::new((self.x, self.y), (self.x, self.y + T::infinity()));
        ls.lines().filter_map(|l| isect.intersection(&l)).count() % 2 == 1
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

        // println!("isect: {a:?} -> {b:?}");

        if !a.is_ortho() {
            panic!("non-orthogonal A");
        }

        // If A is not vertical line, invert axes
        if !a.is_vertical() {
            // println!("invert");
            return a
                .swap_axes()
                .intersection(&b.swap_axes())
                .and_then(|p| Some(p.yx()));
        }

        // Get X-axis differences
        let c = Line::new(b.start, a.start);
        let dx_c = c.dx();
        let dx_b = b.dx();

        // println!("dx_c={dx_c:?}, dx_b={dx_b:?}");

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

        // println!("slope_b={b:?}, slope_c={c:?}, slope_d={slope_d:?}");
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
