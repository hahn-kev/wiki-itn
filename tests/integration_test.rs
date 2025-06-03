use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command; // Added for Command
use std::str; // Added for converting output bytes to str

use wiki_itn::process_html;

const TEST_URL: &str = "https://en.wikipedia.org/wiki/Template:In_the_news";
const OUTPUT_FILE: &str = "feed.xml";

#[test]
fn fetch_itn_and_generate_feed() {
    // 1. Fetch the HTML content using curl
    let output = Command::new("curl")
        .arg("-sSLf") // -s (silent), -S (show error), -L (follow redirects), -f (fail fast)
        .arg(TEST_URL)
        .output()
        .expect("Failed to execute curl command");

    assert!(output.status.success(), "curl command failed with status: {}", output.status);

    let html_content = match str::from_utf8(&output.stdout) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    assert!(!html_content.is_empty(), "Fetched HTML content is empty");

    // Print the HTML content for debugging
    // println!("Fetched HTML content by curl:\n{}", html_content); // Commented out for now

    // 2. Call the process_html function
    let feed_xml = process_html(html_content);
    assert!(!feed_xml.is_empty(), "Generated feed XML is empty");

    // 3. Write the output to feed.xml
    let output_path = Path::new(OUTPUT_FILE);
    let mut file = File::create(output_path).expect("Failed to create feed.xml");
    file.write_all(feed_xml.as_bytes()).expect("Failed to write to feed.xml");

    // 4. Assert that the file was created and is not empty
    assert!(output_path.exists(), "feed.xml was not created");
    let metadata = fs::metadata(output_path).expect("Failed to get metadata for feed.xml");
    assert!(metadata.len() > 0, "feed.xml is empty");

    // Optional: Clean up
    // fs::remove_file(output_path).expect("Failed to remove feed.xml");
}
