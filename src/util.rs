use geo_types::{Coord, CoordFloat, Line, LineString};
use log::debug;

use crate::rect::Rect;

#[inline]
pub(crate) fn segments_to_linestring<T: CoordFloat>(mut segments: Vec<Line<T>>) -> LineString<T> {
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

#[inline]
pub(crate) fn find_coord_inside<'a, T: CoordFloat>(
    ls: &'a LineString<T>,
    rect: &Rect<T>,
) -> Option<&'a Coord<T>> {
    ls.0.iter().find(|c| rect.coord_inside(c))
}

pub(crate) fn print_queue<T: CoordFloat>(queue: &Vec<(f64, LineString<T>)>) {
    for (p_idx, ls) in queue.into_iter().rev() {
        debug!("p_idx={p_idx}, {ls:?}");
    }
}

#[inline(always)]
pub fn rough_eq<T: CoordFloat>(a: T, b: T) -> bool {
    (a - b).abs() <= T::from(0.00001).unwrap()
}
