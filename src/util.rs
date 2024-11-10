use geo_types::{Coord, CoordFloat, Line, LineString};

#[inline]
pub fn coords_to_lines<T: CoordFloat>(coords: Vec<Coord<T>>) -> Vec<Line<T>> {
    let mut lines = vec![];

    if coords.len() > 1 {
        for i in 0..coords.len() - 1 {
            lines.push(Line::new(coords[i], coords[i + 1]));
        }
    }

    lines
}

#[inline]
pub fn segments_to_linestring<T: CoordFloat>(mut segments: Vec<Line<T>>) -> LineString<T> {
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
