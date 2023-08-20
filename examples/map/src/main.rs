use data::Record;
use rustifact::Map;

rustifact::use_symbols!(RECORD_MAP);

fn main() {
    println!("map len: {}", RECORD_MAP.len());
    println!("first: {:?}", RECORD_MAP.get("first").unwrap());
    println!("second: {:?}", RECORD_MAP.get("second").unwrap());
    println!("third: {:?}", RECORD_MAP.get("third").unwrap());
}
