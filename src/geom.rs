use num_traits::Float;

#[derive(PartialEq, Clone, Debug)]
pub struct Point<T: Float>(pub T, pub T);

impl<T: Float> Point<T> {
    fn slope(&self, other: &Self) -> T {
        (self.1 - other.1) / (other.0 - self.0)
    }

    fn yx(&self) -> Self {
        Self(self.1, self.0)
    }

    pub fn dist_manhattan(&self, other: &Self) -> T {
        (self.0 - other.0).abs() + (self.1 - other.1).abs()
    }
}

impl<T: Float> From<(T, T)> for Point<T> {
    fn from(p: (T, T)) -> Self {
        Point(p.0, p.1)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Segment<T: Float>(pub Point<T>, pub Point<T>);

impl<T: Float + std::fmt::Debug> Segment<T> {
    pub fn new(a: impl Into<Point<T>>, b: impl Into<Point<T>>) -> Self {
        Self(a.into(), b.into())
    }

    // Checks if line B intersects A, where A is orthogonal to X axis (vertical line).
    // If the two lines share a same point, the result is None since clipping is not needed.
    pub fn isect(&self, b: &Self) -> Option<Point<T>> {
        let a = self;

        if !a.is_ortho() {
            panic!("non-orthogonal A");
        }

        // If A is not vertical line, invert axes
        if !a.is_vertical() {
            return a
                .swap_axes()
                .isect(&b.swap_axes())
                .and_then(|p| Some(p.yx()));
        }

        // Get X-axis differences
        let diff_a = b.0 .0 - a.0 .0;
        let diff_b = b.0 .0 - b.1 .0;

        // println!("diff_a={diff_a:?}, diff_b={diff_b:?}");

        // Check diff signatures
        if diff_a.is_sign_positive() != diff_b.is_sign_positive() {
            return None;
        }

        // Check if B is directed toward A
        let slope_a = b.0.slope(&a.0);
        let slope_b = b.0.slope(&a.1);
        let slope_c = b.0.slope(&b.1);

        // println!("slope_a={slope_a:?}, slope_b={slope_b:?}, slope_c={slope_c:?}");

        if slope_c < slope_a.min(slope_b) || slope_c > slope_a.max(slope_b) {
            return None;
        }

        // X-axis distance check
        if diff_b.abs() <= diff_a.abs() {
            return None;
        }

        Some(Point(b.0 .0 - diff_a, b.0 .1 + diff_a * slope_c))
    }

    fn is_vertical(&self) -> bool {
        self.0 .0 == self.1 .0
    }

    fn is_ortho(&self) -> bool {
        self.0 .0 == self.1 .0 || self.0 .1 == self.1 .1
    }

    fn swap_axes(&self) -> Self {
        Self(self.0.yx(), self.1.yx())
    }

    #[allow(dead_code)]
    fn reverse(self) -> Self {
        Self(self.1, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horizontal() {
        //  |
        // -x-
        //  |
        let a = Segment::<f32>::new((0.0, -4.0), (0.0, 4.0));
        let b = Segment::<f32>::new((1.0, 0.0), (-1.0, 0.0));
        assert_eq!(a.isect(&b), Some(Point(0.0, 0.0)));
        assert_eq!(a.isect(&b.reverse()), Some(Point(0.0, 0.0)));

        //  |/
        //  x
        // /|
        let a = Segment::new((0.0, 0.0), (0.0, 4.0));
        let b = Segment::new((-1.0, 0.0), (1.0, 4.0));
        assert_eq!(a.isect(&b), Some(Point(0.0, 2.0)));
        assert_eq!(a.isect(&b.reverse()), Some(Point(0.0, 2.0)));

        //   /
        // -x---
        // /
        let a = Segment::new((0.0, 0.0), (4.0, 0.0));
        let b = Segment::new((4.0, 1.0), (0.0, -1.0));
        assert!(a.isect(&b).is_some());
        assert!(a.isect(&b.reverse()).is_some());

        //    |
        // ---x
        //
        let a = Segment::new((0.0, 0.0), (4.0, 0.0));
        let b = Segment::new((4.0, 4.0), (4.0, 0.0));
        assert!(!a.isect(&b).is_some());
        assert!(!a.isect(&b.reverse()).is_some());

        // Non-intersecting tests
        let a = Segment::new((0.0, 0.0), (0.0, 4.0));
        let b = Segment::new((1.0, 1.0), (0.1, 1.0));
        assert!(!a.isect(&b).is_some());
        assert!(!a.isect(&b.reverse()).is_some());

        let a = Segment::new((0.0, 0.0), (0.0, 4.0));
        let b = Segment::new((1.0, 1.0), (4.0, 4.0));
        assert!(!a.isect(&b).is_some());
        assert!(!a.isect(&b.reverse()).is_some());
    }
}
