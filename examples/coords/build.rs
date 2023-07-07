use data::{Cartesian2D, PlanarCoord, Polar};
use rustifact::ToTokenStream;

fn c2(x: f64, y: f64) -> PlanarCoord {
    PlanarCoord::C(Cartesian2D { x, y })
}

fn p(r: f64, theta: f64) -> PlanarCoord {
    PlanarCoord::P(Polar { r, theta })
}

fn main() {
    let coords = vec![
        c2(1.0, 1.0),
        c2(2.5, 4.6),
        c2(-1.2, 3.0),
        c2(3.6, 2.8),
        c2(1.3, -4.7),
        c2(-5.0, 3.2),
        c2(2.1, 2.1),
        p(2.5, 0.78),
        p(1.5, 2.36),
        p(3.5, 4.71),
        p(2.2, 3.93),
        p(4.4, 5.50),
    ];
    rustifact::write_static_array!(COORDS, PlanarCoord, &coords);
}
