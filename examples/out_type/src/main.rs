use data::{StructUniform, StructVarying};

rustifact::use_symbols!(STRUCT_VARYING, STRUCT_UNIFORM);

fn main() {
    println!("{} {}", STRUCT_VARYING.s, STRUCT_VARYING.num);
    println!("{} {}", STRUCT_UNIFORM.x, STRUCT_UNIFORM.y);
}
