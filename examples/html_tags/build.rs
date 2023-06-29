use rustifact::ToTokenStream;

fn get_non_closable_tags() -> Vec<String> {
    vec!["br", "hr", "img", "input"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn get_closable_tags() -> Vec<String> {
    let mut tag_ids: Vec<String> = vec![
        "html", "head", "title", "body", "p", "a", "div", "span", "ul", "ol", "li", "table", "tr",
        "th", "td", "form", "button", "select", "option", "label", "textarea", "header", "footer",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for i in 1..=6 {
        tag_ids.push(format!("h{}", i));
    }
    tag_ids
}

fn main() {
    let mut tags = Vec::new();
    for tag_id in get_closable_tags()
        .iter()
        .chain(get_non_closable_tags().iter())
    {
        tags.push((
            format!("OPEN_{}", tag_id.to_ascii_uppercase()),
            format!("<{}>", tag_id),
        ));
    }
    for tag_id in get_closable_tags().iter() {
        tags.push((
            format!("CLOSE_{}", tag_id.to_ascii_uppercase()),
            format!("</{}>", tag_id),
        ));
    }
    rustifact::write_statics!(public, HTML_TAGS, &'static str, &tags);
}
