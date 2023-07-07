use rustifact::ToTokenStream;

#[derive(ToTokenStream)]
pub struct Cartesian2D {
    pub x: f64,
    pub y: f64,
}

#[derive(ToTokenStream)]
pub struct Polar {
    pub r: f64,
    pub theta: f64,
}

#[derive(ToTokenStream)]
pub enum PlanarCoord {
    C(Cartesian2D),
    P(Polar),
}
