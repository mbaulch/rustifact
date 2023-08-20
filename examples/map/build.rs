use data::Record;
use rustifact::{MapBuilder, ToTokenStream};

fn main() {
    let mut map = MapBuilder::new();
    map.entry("first", Record { n: 0, s: "abc" });
    map.entry("second", Record { n: 1, s: "def" });
    map.entry("third", Record { n: 2, s: "ghi" });
    rustifact::write_static!(RECORD_MAP, Map<&'static str, Record>, &map);
}
