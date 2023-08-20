use rustifact::ToTokenStream;

#[derive(ToTokenStream, Debug)]
pub struct Record {
    pub n: u32,
    pub s: &'static str,
}
