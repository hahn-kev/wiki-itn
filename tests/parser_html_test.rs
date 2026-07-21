use std::fs;
use std::path::Path;

use wiki_itn::process_html;

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn parses_legacy_flat_ul() {
    let feed = process_html(&fixture("legacy_flat_ul.html"));
    assert!(
        feed.contains("<title>Jiang Zemin</title>"),
        "feed was:\n{}",
        feed
    );
    assert!(feed.contains("href=\"https://en.wikipedia.org/wiki/Jiang_Zemin\""));
    assert!(feed.contains("<title>Example Story</title>"));
    // Footer-only links must not become entries
    assert!(!feed.contains("<title>Other</title>"), "feed was:\n{}", feed);
}

#[test]
fn parses_parsoid_nested_ul_with_absolute_hrefs() {
    let feed = process_html(&fixture("parsoid_nested_ul.html"));
    assert!(
        feed.contains("<title>Andy Burnham</title>"),
        "feed was:\n{}",
        feed
    );
    assert!(feed.contains("href=\"https://en.wikipedia.org/wiki/Andy_Burnham\""));
    assert!(feed.contains("<title>Example Story</title>"));
    assert!(!feed.contains("<title>Other</title>"), "feed was:\n{}", feed);
    // Absolute hrefs must not be double-prefixed
    assert!(!feed.contains("https://en.wikipedia.orghttps://"));
}
