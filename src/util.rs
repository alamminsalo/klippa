use geo_types::{CoordFloat, Line, LineString};

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

pub(crate) fn print_queue<T: CoordFloat>(queue: &Vec<(f64, LineString<T>)>) {
    for (p_idx, ls) in queue.into_iter().rev() {
        println!("p_idx={p_idx}, {ls:?}");
    }
}
