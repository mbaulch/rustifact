rustifact::use_symbols!(CITY_DATA);

fn main() {
    for (name, population) in CITY_DATA.iter() {
        println!("{} has population {}", name, population)
    }
}
