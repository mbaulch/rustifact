use rustifact::ToTokenStream;

fn generate_city_data() -> Vec<(String, u32)> {
    let mut city_data: Vec<(String, u32)> = Vec::new();
    for i in 1..=100 {
        let city_name = format!("City{}", i);
        let population = i * 1000; // Dummy population data
        city_data.push((city_name, population));
    }
    city_data
}

fn main() {
    let city_data = generate_city_data();
    //
    // Let's make city_data accessible from the main crate. We'll write it to
    // a static array CITY_DATA where the type of each element is (&'static str, u32).
    // Note that Strings are converted to static string slices by default.
    //
    rustifact::write_static_array!(CITY_DATA, (&'static str, u32), &city_data);
    //
    // We could have specified the dimension like so:
    //rustifact::write_static_array!(CITY_DATA, (&'static str, u32) : 1, &city_data);
    //
    // When the dimension is unspecified (as above) the default is dimension 1.
}
