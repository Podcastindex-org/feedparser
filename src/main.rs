use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::path::{PathBuf};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use serde::Serialize;
use serde_json::Value as JsonValue;
use xml::reader::{EventReader, XmlEvent};

mod parser_state;
mod tags;
mod outputs;
use parser_state::ParserState;

// Global counter initialized to zero at program start
pub(crate) static GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);
// Per-run output subfolder based on startup UNIX timestamp
pub(crate) static OUTPUT_SUBDIR: OnceLock<PathBuf> = OnceLock::new();

#[derive(Serialize)]
pub struct SqlInsert {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<JsonValue>,
    pub feed_id: Option<i64>,
}

#[tokio::main]
async fn main() {
    // Track total runtime for the entire program
    let program_start = Instant::now();
    // Establish a stable per-run timestamped subfolder under outputs
    let startup_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let subfolder = PathBuf::from("outputs").join(startup_unix.to_string());
    if let Err(e) = fs::create_dir_all(&subfolder) {
        eprintln!("Failed to create outputs subfolder '{}': {}", subfolder.display(), e);
    }
    let _ = OUTPUT_SUBDIR.set(subfolder);

    // Find all XML input files in the inputs directory
    let feeds_dir = "inputs";
    let entries = match fs::read_dir(feeds_dir) {
        Ok(it) => it,
        Err(e) => {
            eprintln!("Unable to read directory '{}': {}", feeds_dir, e);
            return;
        }
    };

    //Process each XML input file in parallel (asynchronously)
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading a directory entry: {}", e);
                continue;
            }
        };

        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>")
            .to_string();

        // Only process regular files with .xml or .txt extension
        let is_file = entry
            .file_type()
            .map(|t| t.is_file())
            .unwrap_or(false);
        let ext_ok = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| {
                let e = e.to_ascii_lowercase();
                e == "xml" || e == "txt"
            })
            .unwrap_or(false);

        if !is_file || !ext_ok {
            continue;
        }

        // Try to parse feed_id from file name pattern: [feed id]_[http response code].txt
        let feed_id: Option<i64> = {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let mut parts = stem.splitn(2, '_');
            let id_part = parts.next().unwrap_or("");
            match id_part.parse::<i64>() {
                Ok(v) => Some(v),
                Err(_) => None,
            }
        };

        match File::open(&path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                // Measure processing time per file
                let start = Instant::now();
                // Run feed processing asynchronously
                process_feed(reader, file_name.clone(), feed_id).await;
                println!("Processed {} in {:?}", file_name, start.elapsed());
            }
            Err(e) => {
                eprintln!("Unable to open file '{}': {}", path.display(), e);
                continue;
            }
        }
    }

    // Print total runtime just before exiting
    println!("Total runtime: {:?}", program_start.elapsed());
}

// Synchronous parser implementation (unchanged logic)
fn process_feed_sync<R: Read>(reader: R, _source_name: &str, feed_id: Option<i64>) {
    // Wrap in a BufReader so we can read header lines and then pass the same reader to the XML parser
    let mut buf_reader = BufReader::new(reader);

    // New input format header (first 4 lines before the XML):
    // 1) unix timestamp of Last-Modified
    // 2) e-tag header (or [[NO_ETAG]])
    // 3) XML feed URL
    // 4) unix timestamp of when the XML was downloaded
    // 5..end) the XML document

    fn read_line_trim<R: Read>(r: &mut BufReader<R>) -> Option<String> {
        let mut line = String::new();
        match r.read_line(&mut line) {
            Ok(0) => None, // EOF
            Ok(_) => Some(line.trim_end_matches(['\r', '\n']).to_string()),
            Err(_) => None,
        }
    }

    let last_modified_str = read_line_trim(&mut buf_reader);
    let etag_str = read_line_trim(&mut buf_reader);
    let feed_url_str = read_line_trim(&mut buf_reader);
    let downloaded_str = read_line_trim(&mut buf_reader);

    // Parse optional metadata (currently not used in SQL output; reserved for future use)
    let _last_modified_unix: Option<i64> = last_modified_str
        .as_deref()
        .and_then(|s| s.parse::<i64>().ok());
    let _etag_opt: Option<String> = etag_str.as_deref().and_then(|s| {
        if s == "[[NO_ETAG]]" || s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    });
    let _feed_url_opt: Option<String> = feed_url_str.filter(|s| !s.is_empty());
    let _downloaded_unix: Option<i64> = downloaded_str
        .as_deref()
        .and_then(|s| s.parse::<i64>().ok());

    // After headers, read the remaining payload to determine if XML content exists
    let mut xml_bytes: Vec<u8> = Vec::new();
    if let Err(e) = buf_reader.read_to_end(&mut xml_bytes) {
        eprintln!("Failed to read XML payload after headers: {}", e);
        return;
    }

    // Check if payload is empty or whitespace-only
    let has_non_whitespace = xml_bytes
        .iter()
        .any(|b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n'));

    if !has_non_whitespace {
        // XML payload is empty or whitespace-only: write a single channel-level record with blank fields
        let record = SqlInsert {
            table: "newsfeeds".to_string(),
            columns: vec![
                "feed_id".to_string(),
                "title".to_string(),
                "link".to_string(),
                "description".to_string(),
                "generator".to_string(),
                "itunes_author".to_string(),
            ],
            values: vec![
                match feed_id { Some(v) => JsonValue::from(v), None => JsonValue::Null },
                JsonValue::from(String::new()),
                JsonValue::from(String::new()),
                JsonValue::from(String::new()),
                JsonValue::from(String::new()),
                JsonValue::from(String::new()),
            ],
            feed_id,
        };

        // Use per-run outputs subfolder established at startup
        let out_dir: PathBuf = OUTPUT_SUBDIR
            .get()
            .cloned()
            .unwrap_or_else(|| PathBuf::from("outputs"));
        if let Err(e) = fs::create_dir_all(&out_dir) {
            eprintln!("Failed to create outputs directory '{}': {}", out_dir.display(), e);
        }

        // Compute counter (1-based) and build filename
        let counter_val = GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
        let fid_for_name = feed_id
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string());
        let file_name = format!("{}_{}_{}.json", counter_val, "newsfeeds", fid_for_name);
        let file_path = out_dir.join(file_name);

        match serde_json::to_string(&record) {
            Ok(serialized) => {
                if let Err(e) = fs::write(&file_path, serialized) {
                    eprintln!("Failed to write {}: {}", file_path.display(), e);
                } else {
                    println!(
                        "Empty XML payload detected for '{}'; wrote blank newsfeeds record to {}",
                        _source_name,
                        file_path.display()
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize blank newsfeeds record: {}", e);
            }
        }

        // Nothing else to do for this file
        return;
    }

    // Create an XML parser from the buffered payload
    let cursor = Cursor::new(xml_bytes);
    let parser = EventReader::new(cursor);

    // Parser state holds all flags and accumulators used by handlers
    let mut state = ParserState::default();

    // Parse the XML document
    for event in parser {
        match event {
            //A tag is opened.
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                state.current_element = name.local_name.clone();
                tags::dispatch_start(&name, &attributes, &mut state);
            }

            //Text is found.
            Ok(XmlEvent::Characters(data)) => {
                let current = state.current_element.clone();
                tags::dispatch_text(&current, &data, &mut state);
            }

            // CDATA is also textual content — treat it the same as Characters
            Ok(XmlEvent::CData(data)) => {
                let current = state.current_element.clone();
                tags::dispatch_text(&current, &data, &mut state);
            }

            //A tag is closed.
            Ok(XmlEvent::EndElement { name }) => {
                tags::dispatch_end(&name, feed_id, &mut state);
            }

            //An error occurred.
            Err(e) => {
                eprintln!("Error parsing XML: {}", e);
                break;
            }
            _ => {}
        }
    }
}

// Public async wrapper that executes the synchronous parser on a blocking thread
async fn process_feed<R>(reader: R, source_name: String, feed_id: Option<i64>)
where
    R: Read + Send + 'static,
{
    // Ignore the JoinError here but log if it occurs
    let source_for_task = source_name.clone();
    if let Err(e) = tokio::task::spawn_blocking(move || {
        process_feed_sync(reader, &source_for_task, feed_id);
    })
        .await
    {
        eprintln!("Error in async processing for '{}': {}", source_name, e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let base = std::env::temp_dir();
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let dir = base.join(format!("feedparser_test_{}", ts));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn ensure_output_dir() -> PathBuf {
        if let Some(existing) = OUTPUT_SUBDIR.get() {
            return existing.clone();
        }
        let dir = unique_temp_dir();
        let _ = OUTPUT_SUBDIR.set(dir.clone());
        dir
    }

    #[test]
    fn writes_channel_title_to_newsfeeds_output() {
        // Arrange: ensure outputs directory is set once for all tests in this process
        let out_dir = ensure_output_dir();

        // Synthetic input: 4 header lines followed by minimal RSS with channel title
        let last_modified = "0"; // placeholder
        let etag = "[[NO_ETAG]]";
        let url = "https://example.com/feed.xml";
        let downloaded = "0";
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>My Test Channel</title>
  </channel>
</rss>"#;

        let input = format!(
            "{last}\n{etag}\n{url}\n{dl}\n{xml}\n",
            last = last_modified,
            etag = etag,
            url = url,
            dl = downloaded,
            xml = xml
        );

        let feed_id = Some(424242_i64);

        // Act: process the synthetic feed synchronously
        process_feed_sync(Cursor::new(input.into_bytes()), "<test>", feed_id);

        // Assert: a newsfeeds JSON file exists with the expected title
        let entries = fs::read_dir(&out_dir)
            .expect("output directory should be readable");
        let mut found_path: Option<PathBuf> = None;
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.contains("_newsfeeds_") && name.ends_with("424242.json") {
                    found_path = Some(path);
                    break;
                }
            }
        }

        let file_path = found_path.expect("should have written a newsfeeds output file");
        let contents = fs::read_to_string(&file_path)
            .expect("should be able to read newsfeeds file");
        let v: serde_json::Value = serde_json::from_str(&contents)
            .expect("valid JSON in newsfeeds file");

        // Basic shape assertions
        assert_eq!(v["table"], "newsfeeds");
        assert_eq!(v["columns"][1], "title");
        assert_eq!(v["feed_id"], serde_json::json!(424242));

        // Channel title should be the second value (index 1), trimmed
        assert_eq!(v["values"][1], serde_json::json!("My Test Channel"));
    }

    #[test]
    fn writes_channel_link_and_description_cdata() {
        // Arrange
        let out_dir = ensure_output_dir();

        let last_modified = "0";
        let etag = "[[NO_ETAG]]";
        let url = "https://example.com/feed.xml";
        let downloaded = "0";
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Channel With Links</title>
    <link>https://example.com/</link>
    <description><![CDATA[ This is a <b>CDATA</b> description. ]]></description>
  </channel>
</rss>"#;

        let input = format!(
            "{last}\n{etag}\n{url}\n{dl}\n{xml}\n",
            last = last_modified,
            etag = etag,
            url = url,
            dl = downloaded,
            xml = xml
        );

        let feed_id = Some(777001_i64);

        // Act
        process_feed_sync(Cursor::new(input.into_bytes()), "<test>", feed_id);

        // Assert: find the newsfeeds file for this feed_id
        let entries = fs::read_dir(&out_dir).expect("output directory should be readable");
        let mut found_path: Option<PathBuf> = None;
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.contains("_newsfeeds_") && name.ends_with("777001.json") {
                    found_path = Some(path);
                    break;
                }
            }
        }

        let file_path = found_path.expect("should have written a newsfeeds output file");
        let contents = fs::read_to_string(&file_path).expect("read newsfeeds file");
        let v: serde_json::Value = serde_json::from_str(&contents).expect("valid JSON");

        assert_eq!(v["table"], "newsfeeds");
        // title
        assert_eq!(v["values"][1], serde_json::json!("Channel With Links"));
        // link
        assert_eq!(v["values"][2], serde_json::json!("https://example.com/"));
        // description (trimmed)
        assert_eq!(v["values"][3], serde_json::json!("This is a <b>CDATA</b> description."));
    }

    #[test]
    fn writes_item_title_link_description_with_cdata() {
        // Arrange
        let out_dir = ensure_output_dir();

        let last_modified = "0";
        let etag = "[[NO_ETAG]]";
        let url = "https://example.com/feed.xml";
        let downloaded = "0";
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Channel</title>
    <item>
      <title>Episode 1</title>
      <link>https://example.com/ep1</link>
      <description><![CDATA[ Hello & welcome! ]]></description>
    </item>
  </channel>
</rss>"#;

        let input = format!(
            "{last}\n{etag}\n{url}\n{dl}\n{xml}\n",
            last = last_modified,
            etag = etag,
            url = url,
            dl = downloaded,
            xml = xml
        );

        let feed_id = Some(777002_i64);

        // Act
        process_feed_sync(Cursor::new(input.into_bytes()), "<test>", feed_id);

        // Assert: find the nfitems file for this feed_id
        let entries = fs::read_dir(&out_dir).expect("output directory should be readable");
        let mut found_path: Option<PathBuf> = None;
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.contains("_nfitems_") && name.ends_with("777002.json") {
                    found_path = Some(path);
                    break;
                }
            }
        }

        let file_path = found_path.expect("should have written an nfitems output file");
        let contents = fs::read_to_string(&file_path).expect("read nfitems file");
        let v: serde_json::Value = serde_json::from_str(&contents).expect("valid JSON");

        assert_eq!(v["table"], "nfitems");
        assert_eq!(v["values"][1], serde_json::json!("Episode 1"));
        assert_eq!(v["values"][2], serde_json::json!("https://example.com/ep1"));
        assert_eq!(v["values"][3], serde_json::json!("Hello & welcome!"));
    }

    #[test]
    fn writes_channel_itunes_author_to_newsfeeds_output() {
        // Arrange
        let out_dir = ensure_output_dir();

        let last_modified = "0";
        let etag = "[[NO_ETAG]]";
        let url = "https://example.com/feed.xml";
        let downloaded = "0";
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
  <channel>
    <title>Channel With iTunes</title>
    <itunes:author>  ACME Media  </itunes:author>
  </channel>
</rss>"#;

        let input = format!(
            "{last}\n{etag}\n{url}\n{dl}\n{xml}\n",
            last = last_modified,
            etag = etag,
            url = url,
            dl = downloaded,
            xml = xml
        );

        let feed_id = Some(808001_i64);

        // Act
        process_feed_sync(Cursor::new(input.into_bytes()), "<test>", feed_id);

        // Assert: find the newsfeeds file for this feed_id
        let entries = fs::read_dir(&out_dir).expect("output directory should be readable");
        let mut found_path: Option<PathBuf> = None;
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.contains("_newsfeeds_") && name.ends_with("808001.json") {
                    found_path = Some(path);
                    break;
                }
            }
        }

        let file_path = found_path.expect("should have written a newsfeeds output file");
        let contents = fs::read_to_string(&file_path).expect("read newsfeeds file");
        let v: serde_json::Value = serde_json::from_str(&contents).expect("valid JSON");

        assert_eq!(v["table"], "newsfeeds");
        // itunes_author is the 6th value (index 5) and should be trimmed
        assert_eq!(v["values"][5], serde_json::json!("ACME Media"));
    }

    #[test]
    fn writes_item_itunes_author_to_nfitems_output() {
        // Arrange
        let out_dir = ensure_output_dir();

        let last_modified = "0";
        let etag = "[[NO_ETAG]]";
        let url = "https://example.com/feed.xml";
        let downloaded = "0";
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
  <channel>
    <title>Channel</title>
    <item>
      <title>Episode A</title>
      <itunes:author> Guest Speaker </itunes:author>
    </item>
  </channel>
</rss>"#;

        let input = format!(
            "{last}\n{etag}\n{url}\n{dl}\n{xml}\n",
            last = last_modified,
            etag = etag,
            url = url,
            dl = downloaded,
            xml = xml
        );

        let feed_id = Some(808002_i64);

        // Act
        process_feed_sync(Cursor::new(input.into_bytes()), "<test>", feed_id);

        // Assert: find the nfitems file for this feed_id
        let entries = fs::read_dir(&out_dir).expect("output directory should be readable");
        let mut found_path: Option<PathBuf> = None;
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.contains("_nfitems_") && name.ends_with("808002.json") {
                    found_path = Some(path);
                    break;
                }
            }
        }

        let file_path = found_path.expect("should have written an nfitems output file");
        let contents = fs::read_to_string(&file_path).expect("read nfitems file");
        let v: serde_json::Value = serde_json::from_str(&contents).expect("valid JSON");

        assert_eq!(v["table"], "nfitems");
        // itunes_author is the 7th value (index 6) for nfitems
        assert_eq!(v["values"][6], serde_json::json!("Guest Speaker"));
    }

    #[test]
    fn writes_channel_generator_to_newsfeeds_output() {
        // Arrange
        let out_dir = ensure_output_dir();

        let last_modified = "0";
        let etag = "[[NO_ETAG]]";
        let url = "https://example.com/feed.xml";
        let downloaded = "0";
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Channel Gen</title>
    <generator> WordPress 6.5.2 </generator>
  </channel>
</rss>"#;

        let input = format!(
            "{last}\n{etag}\n{url}\n{dl}\n{xml}\n",
            last = last_modified,
            etag = etag,
            url = url,
            dl = downloaded,
            xml = xml
        );

        let feed_id = Some(900901_i64);

        // Act
        process_feed_sync(Cursor::new(input.into_bytes()), "<test>", feed_id);

        // Assert: find the newsfeeds file for this feed_id
        let entries = fs::read_dir(&out_dir).expect("output directory should be readable");
        let mut found_path: Option<PathBuf> = None;
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.contains("_newsfeeds_") && name.ends_with("900901.json") {
                    found_path = Some(path);
                    break;
                }
            }
        }

        let file_path = found_path.expect("should have written a newsfeeds output file");
        let contents = fs::read_to_string(&file_path).expect("read newsfeeds file");
        let v: serde_json::Value = serde_json::from_str(&contents).expect("valid JSON");

        assert_eq!(v["table"], "newsfeeds");
        // generator is the 5th value (index 4) and should be trimmed
        assert_eq!(v["values"][4], serde_json::json!("WordPress 6.5.2"));
    }
}