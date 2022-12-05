extern crate core;

use std::collections::HashMap;
use std::io::{Read, stdin};

use chrono::{Duration, SecondsFormat, Utc};
use html_parser::{Dom, Element, Node};

use news_item::NewsItem;

mod news_item;

fn main() {
    process();
}

const URL_PREFIX: &str = "https://en.wikipedia.org";
const FEED_URL: &str = "https://en.wikipedia.org/wiki/Template:In_the_news";

// expecting value from: https://en.wikipedia.org/wiki/Template:In_the_news
fn process() {
    let mut piped_in_value = String::new();
    stdin().read_to_string(&mut piped_in_value).expect("Value of ITN page must be piped in");
    let dom = Dom::parse(piped_in_value.as_str()).expect("Failed to parse document");
    let root_element = find_root_element(&dom).expect("Unable to find root element ul");
    let mut list: Vec<NewsItem> = Vec::with_capacity(root_element.children.len());
    for node in &root_element.children {
        match node {
            Node::Element(e) if e.name == "li" => list.push(element_to_news_item(e)),
            _ => ()
        }
    }
    let feed_text = write_atom_feed(list);
    print!("{}", feed_text);
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
    return format!(r##"<?xml version="#1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <title>Wikipedia In The News</title>
    <link href="{FEED_URL}"/>
    <id>urn:uuid:e0579856-2b18-4a8e-8c76-771c24206362</id>
    <updated>{time}</updated>{entries}
</feed>
"##);
}


fn element_to_news_item(e: &Element) -> NewsItem {
    let bold_element = find_element(&e.children, "b").expect("unable to find bold link");
    let first_node = bold_element.children.first();
    let link_element = (match first_node {
        Some(Node::Element(e)) if e.name == "a" => Some(e),
        _ => None
    }).expect("bold element is empty");
    let mut link = link_element.attributes.get("href").expect("link should have href")
        .as_ref().expect("href should not be empty").clone();
    let title = link_element.attributes.get("title").expect("link should have title")
        .as_ref().expect("title should not be empty").clone();
    let id = link.clone();
    link.insert_str(0, URL_PREFIX);

    NewsItem {
        title,
        body: element_children_to_string(e),
        url: link,
        id,
    }
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
    let body_content = find_element_with_id(&dom.children, "div", "body_content")
        .expect("unable to find div id='body_content'");
    let div = find_element_with_class(&body_content.children, "div", &"mw-parser-output".to_string())
        .expect("unable to find div class='mw-parser-output'");
    for node in &div.children {
        match node {
            Node::Element(e) if e.name == "ul" => return Some(e),
            _ => continue
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
