rustifact::use_symbols!(ARRAY_4D);

fn main() {
    // Use the 4D array
    for i in 0..5 {
        for j in 0..5 {
            for k in 0..5 {
                if k != 0 {
                    print!(" ");
                }
                for l in 0..5 {
                    if l != 0 {
                        print!(",");
                    }
                    print!("{}", ARRAY_4D[i][j][k][l]);
                }
            }
            println!("");
        }
        println!("\n");
    }
}
