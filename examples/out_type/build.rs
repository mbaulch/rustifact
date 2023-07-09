use data::{StructUniform, StructVaryingIn};
use rustifact::ToTokenStream;

fn main() {
    rustifact::write_static!(
        STRUCT_VARYING,
        StructVarying,
        &StructVaryingIn {
            s: "hello".to_string(),
            num: 5,
        }
    );
    rustifact::write_static!(
        STRUCT_UNIFORM,
        StructUniform,
        &StructUniform { x: -3, y: 2 }
    );
}
