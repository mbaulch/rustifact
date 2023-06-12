rustifact::use_symbols!(ARRAY_4D);

fn main() {
    // Use the 4D array
    for i in 0..5 {
        for j in 0..5 {
            for k in 0..5 {
                for l in 0..5 {
                    println!("{}", ARRAY_4D[i][j][k][l]);
                }
            }
        }
    }
}
