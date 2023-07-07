use data::{Cartesian2D, PlanarCoord, Polar};

rustifact::use_symbols!(COORDS);

fn main() {
    for coord in COORDS.iter() {
        match coord {
            PlanarCoord::C(Cartesian2D { x, y }) => print!("c({}, {}), ", x, y),
            PlanarCoord::P(Polar { r, theta }) => print!("p({}, {}), ", r, theta),
        }
    }
}
