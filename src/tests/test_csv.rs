use crate::*;
use geo::BoundingRect;
use geo_types::{MultiPolygon, Polygon};
use std::{collections::HashMap, fs::File, sync::OnceLock};
use wkt::{ToWkt, TryFromWkt};

pub fn get_wkt(id: &str) -> (ClipRect<f64>, Geometry) {
    let csv_path = "./assets/test/wkt.csv";

    // Open the CSV file.
    let file = File::open(csv_path).unwrap();
    let mut reader = csv::Reader::from_reader(file);

    // Iterate through each record in the CSV file.
    for result in reader.records() {
        let record = result.unwrap();
        let mut iter = record.iter();
        let key = iter.next().unwrap();

        if key == id {
            let bbox: Geometry = Geometry::try_from_wkt_str(iter.next().unwrap()).unwrap();
            let geom: Geometry = Geometry::try_from_wkt_str(iter.next().unwrap()).unwrap();

            let rect = bbox.bounding_rect().unwrap();
            let clipper = ClipRect::new(rect.min().x, rect.min().y, rect.max().x, rect.max().y);

            return (clipper, geom);
        }
    }

    panic!("not found")
}

#[test]
fn test_osm_1() {
    let (clipper, g) = get_wkt("10412355");

    let g: Polygon = clipper.clip(&g).unwrap().try_into().unwrap();

    // Check inner ring count
    let (_, int) = g.into_inner();
    assert_eq!(int.len(), 1);
}
