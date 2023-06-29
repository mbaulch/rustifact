#[allow(dead_code)]
mod tags {
    rustifact::use_symbols!(HTML_TAGS);
}

fn main() {
    println!("{}", tags::OPEN_HTML);
    println!("{}", tags::OPEN_HEAD);
    println!("{}Title goes here{}", tags::OPEN_TITLE, tags::CLOSE_TITLE);
    println!("{}", tags::CLOSE_HEAD);
    println!("{}", tags::OPEN_BODY);
    println!("{}A heading{}", tags::OPEN_H1, tags::CLOSE_H1);
    println!("{}A paragraph.{}", tags::OPEN_P, tags::CLOSE_P);
    println!("{}", tags::OPEN_UL);
    println!("{}Item 1{}", tags::OPEN_LI, tags::CLOSE_LI);
    println!("{}Item 2{}", tags::OPEN_LI, tags::CLOSE_LI);
    println!("{}Item 3{}", tags::OPEN_LI, tags::CLOSE_LI);
    println!("{}", tags::CLOSE_UL);
    println!("{}", tags::CLOSE_BODY);
    println!("{}", tags::CLOSE_HTML);
}
