use geo_types::{Coord, CoordFloat, Line, LineString};

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
pub(crate) fn find_coord_inside<T: CoordFloat>(
    ls: &LineString<T>,
    rect: &Rect<T>,
) -> Option<Coord<T>> {
    ls.0.iter()
        .find(|c| rect.x0 < c.x && c.x < rect.x1 && rect.y0 < c.y && c.y < rect.y1)
        .copied()
}

#[allow(dead_code)]
pub(crate) fn print_queue<T: CoordFloat>(queue: &Vec<(f64, LineString<T>)>) {
    for (p_idx, ls) in queue.into_iter().rev() {
        println!("p_idx={p_idx}, {ls:?}");
    }
}
