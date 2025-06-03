extern crate core;

use std::collections::HashMap;
use std::io::{Read, stdin};

use chrono::{Duration, SecondsFormat, Utc};
use html_parser::{Dom, Element, Node};

use news_item::NewsItem;

mod news_item;

fn main() {
    let mut piped_in_value = String::new();
    stdin().read_to_string(&mut piped_in_value).expect("Value of ITN page must be piped in");
    let feed_xml = process_html(&piped_in_value); // Call the new function
    print!("{}", feed_xml); // Print the result
}

const URL_PREFIX: &str = "https://en.wikipedia.org";
const FEED_URL: &str = "https://en.wikipedia.org/wiki/Template:In_the_news";

// expecting value from: https://en.wikipedia.org/wiki/Template:In_the_news
pub fn process_html(html_content: &str) -> String {
    let dom = Dom::parse(html_content).expect("Failed to parse document");
    let root_element = find_root_element(&dom).expect("Unable to find root element ul");
    let mut list: Vec<NewsItem> = Vec::new(); // Initialize as empty
    for node in &root_element.children {
        match node {
            Node::Element(e) if e.name == "li" => {
                if let Some(news_item) = element_to_news_item(e) {
                    list.push(news_item);
                } else {
                    // Optionally log skipped item
                    println!("Skipping_li_item_due_to_parsing_issue: {}", element_children_to_string(e));
                }
            }
            _ => ()
        }
    }
    let feed_text = write_atom_feed(list);
    feed_text // Return the feed text instead of printing
}

fn write_atom_feed(list: Vec<NewsItem>) -> String {
    let mut time = Utc::now();

    let mut entries = String::new();
    for NewsItem { title, body, url, id } in list {
        time = time + Duration::seconds(-1);
        let time = time.to_rfc3339_opts(SecondsFormat::Secs, true);
        entries.push_str(&format!(r#"
    <entry>
        <title>{title}</title>
        <link href="{url}"/>
        <id>{id}</id>
        <published>{time}</published>
        <updated>{time}</updated>
        <content type="xhtml"><div xmlns="http://www.w3.org/1999/xhtml">{body}</div></content>
    </entry>"#));
    }

    let time = time.to_rfc3339_opts(SecondsFormat::Secs, true);
    return format!(r##"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <title>Wikipedia In The News</title>
    <link href="{FEED_URL}"/>
    <id>urn:uuid:e0579856-2b18-4a8e-8c76-771c24206362</id>
    <updated>{time}</updated>{entries}
</feed>
"##);
}


fn element_to_news_item(e: &Element) -> Option<NewsItem> {
    let bold_element = find_element(&e.children, "b")?;
    let first_node = bold_element.children.first()?;
    let link_element = match first_node {
        Node::Element(el) if el.name == "a" => Some(el),
        _ => None,
    }?;

    // Ensure attributes exist and have values
    let href_opt = link_element.attributes.get("href")?.as_ref()?;
    let title_opt = link_element.attributes.get("title")?.as_ref()?;

    let mut link = href_opt.clone();
    let title = title_opt.clone();
    let id = link.clone();
    link.insert_str(0, URL_PREFIX);

    Some(NewsItem {
        title,
        body: element_children_to_string(e),
        url: link,
        id,
    })
}

fn element_children_to_string(e: &Element) -> String {
    let mut str = String::new();
    for node in &e.children {
        match node {
            Node::Text(t) => { str.push_str(t) }
            Node::Element(e) => {
                str.push_str(&element_to_string(e))
            }
            _ => {}
        }
    }
    str
}

fn element_to_string(e: &Element) -> String {
    format!("<{}{}>{}</{0}>", e.name, attributes_to_string(&e.attributes), element_children_to_string(e))
}

fn attributes_to_string(attr: &HashMap<String, Option<String>>) -> String {
    let mut str = String::new();
    attr.iter().for_each(|(name, value)| {
        str.push_str(&match value {
            Some(value) if name == "href" => format!(" {}=\"{}{}\"", name, URL_PREFIX, value),
            Some(value) => format!(" {}=\"{}\"", name, value),
            None => format!(" {}", name)
        })
    });
    str
}

fn find_root_element(dom: &Dom) -> Option<&Element> {
    let content_text_div = find_element_with_id(&dom.children, "div", "mw-content-text")?;
    let parser_output_div = find_element_with_class(&content_text_div.children, "div", &"mw-parser-output".to_string())?;
    for node in &parser_output_div.children {
        match node {
            Node::Element(e) if e.name == "ul" => return Some(e),
            _ => continue,
        }
    }
    None
}

#[allow(clippy::ptr_arg)]
fn find_element_with_class<'a>(children: &'a [Node], element_name: &str, class: &String) -> Option<&'a Element> {
    for node in children {
        if let Node::Element(e) = node {
            if e.name == element_name && e.classes.contains(class) { return Some(e); }
            if let Some(e) = find_element_with_class(&e.children, element_name, class) { return Some(e); }
        }
    }
    None
}

#[allow(clippy::ptr_arg)]
fn find_element_with_id<'a>(children: &'a [Node], element_name: &str, id: &str) -> Option<&'a Element> {
    for node in children {
        if let Node::Element(e) = node {
            if e.name == element_name && e.id.as_ref().map_or(false, |i| i == id) { return Some(e); }
            if let Some(e) = find_element_with_id(&e.children, element_name, id) { return Some(e); }
        }
    }
    None
}

fn find_element<'a>(children: &'a [Node], element_name: &str) -> Option<&'a Element> {
    for node in children {
        if let Node::Element(e) = node {
            if e.name == element_name { return Some(e); }
            if let Some(e) = find_element(&e.children, element_name) { return Some(e); }
        }
    }
    None
}
