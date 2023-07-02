use rustifact::ToTokenStream;

fn main() {
    // Generate a 4D array
    let mut arr = [[[[(0 as u32); 5]; 5]; 5]; 5];
    for i in 0..5 {
        for j in 0..5 {
            for k in 0..5 {
                for l in 0..5 {
                    arr[i][j][k][l] = i as u32 + j as u32 + k as u32 + l as u32;
                }
            }
        }
    }
    // Write out the data as a static array
    rustifact::write_static_array!(ARRAY_4D, u32 : 4, arr);
}
