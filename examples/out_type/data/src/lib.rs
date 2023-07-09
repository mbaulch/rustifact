use rustifact::ToTokenStream;

pub struct StructVarying {
    pub s: &'static str,
    pub num: usize,
}

#[derive(ToTokenStream)]
#[OutType(StructVarying)]
pub struct StructVaryingIn {
    pub s: String,
    pub num: usize,
}

#[derive(ToTokenStream)]
pub struct StructUniform {
    pub x: i32,
    pub y: i32,
}
