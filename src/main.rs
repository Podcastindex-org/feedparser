use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::io::Cursor;
use std::sync::atomic::AtomicUsize;
use std::sync::OnceLock;
use std::path::{PathBuf};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use xml::reader::{EventReader, XmlEvent};
use xml::name::OwnedName;

mod parser_state;
mod tags;
mod outputs;
#[cfg(test)]
mod tests;

use parser_state::ParserState;

// Global counter initialized to zero at program start
pub(crate) static GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);
// Per-run output subfolder based on startup UNIX timestamp
pub(crate) static OUTPUT_SUBDIR: OnceLock<PathBuf> = OnceLock::new();

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
        // XML payload is empty or whitespace-only: emit a single newsfeeds row matching partytime shape
        let state = ParserState::default();
        outputs::write_newsfeeds(&state, feed_id);
        return;
    }

    // Create an XML parser from the buffered payload
    let cursor = Cursor::new(xml_bytes);
    let parser = EventReader::new(cursor);

    // Parser state holds all flags and accumulators used by handlers
    let mut state = ParserState::default();

    fn get_prefixed_name(name: &OwnedName) -> String {
        let prefix = name.prefix.clone();
        let local_name = name.local_name.clone();

        if (matches!(prefix.as_deref(), Some("itunes"))
            || matches!(name.namespace.as_deref(), Some("http://www.itunes.com/dtds/podcast-1.0.dtd"))
        ) {
            format!("itunes:{}", local_name)
        } else if (matches!(prefix.as_deref(), Some("podcast"))
            || matches!(name.namespace.as_deref(), Some("https://podcastindex.org/namespace/1.0"))
            || matches!(name.namespace.as_deref(), Some("http://podcastindex.org/namespace/1.0"))
        ) {
            format!("podcast:{}", local_name)
        } else if name.prefix.as_deref() == Some("atom")
            || matches!(name.namespace.as_deref(), Some("http://www.w3.org/2005/Atom")
        ) {
            format!("atom:{}", local_name)
        } else if prefix.is_some() {
            format!("{}:{}", prefix.unwrap(), local_name)
        } else {
            local_name
        }
    }

    // Parse the XML document
    for event in parser {
        match event {
            //A tag is opened.
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                state.current_element = get_prefixed_name(&name);
                let current = state.current_element.clone();
                tags::dispatch_start(&current, &attributes, &mut state);
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
                state.current_element = get_prefixed_name(&name);
                let current = state.current_element.clone();
                tags::dispatch_end(&current, feed_id, &mut state);
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