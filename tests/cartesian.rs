use std::time::{self, SystemTime, UNIX_EPOCH};

use boolygon::{cartesian::Polygon, Shape, Tolerance};
use rand::Rng;

#[test]
#[ignore]
pub fn cartesian() {
    type Sample = [[f64; 2]; 500];
    
    let mut rng = rand::rng();
    let subject = Shape::from(Polygon::from(rng.random::<Sample>().to_vec()));
    let clip = Shape::from(Polygon::from(rng.random::<Sample>().to_vec()));

    println!(">>>>>>>>>>>>>>>>>>>>");
    let start = SystemTime::now();
    subject.not(clip, Tolerance::default());
    let end = SystemTime::now();

    println!("Duration: {} s", end.duration_since(start).unwrap().as_millis());
}