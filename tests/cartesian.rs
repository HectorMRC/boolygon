use std::time::SystemTime;

use boolygon::{cartesian, Shape, Tolerance};
use rand::Rng;

#[test]
#[ignore]
pub fn cartesian() {
    type Sample = [[f64; 2]; 1000];

    let mut rng = rand::rng();
    let subject = Shape::from(cartesian::Polygon::from(rng.random::<Sample>().to_vec()));
    let clip = Shape::from(cartesian::Polygon::from(rng.random::<Sample>().to_vec()));

    let start = SystemTime::now();
    subject.and(clip, Tolerance::default());
    let end = SystemTime::now();

    println!(
        "Duration: {} s",
        end.duration_since(start).unwrap().as_millis()
    );
}
