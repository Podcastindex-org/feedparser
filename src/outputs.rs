use std::fs;
use std::path::PathBuf;

use serde_json::Value as JsonValue;

use crate::{SqlInsert, OUTPUT_SUBDIR, GLOBAL_COUNTER};
use crate::parser_state::ParserState;

fn get_output_dir() -> PathBuf {
    OUTPUT_SUBDIR
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("outputs"))
}

fn write_record(record: &SqlInsert, table_for_name: &str) {
    // Ensure directory exists
    let out_dir = get_output_dir();
    if let Err(e) = fs::create_dir_all(&out_dir) {
        eprintln!("Failed to create outputs directory '{}': {}", out_dir.display(), e);
    }

    // Compute counter (1-based) and build filename
    let counter_val = GLOBAL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    let fid_for_name = record
        .feed_id
        .map(|v| v.to_string())
        .unwrap_or_else(|| "NULL".to_string());
    let file_name = format!("{}_{}_{}.json", counter_val, table_for_name, fid_for_name);
    let file_path = out_dir.join(file_name);

    match serde_json::to_string(record) {
        Ok(serialized) => {
            if let Err(e) = fs::write(&file_path, serialized) {
                eprintln!("Failed to write {}: {}", file_path.display(), e);
            }
        }
        Err(e) => {
            eprintln!("Failed to serialize record for {}: {}", table_for_name, e);
        }
    }
}

pub fn write_newsfeeds(state: &ParserState, feed_id: Option<i64>) {
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
            JsonValue::from(state.channel_title.trim().to_string()),
            JsonValue::from(state.channel_link.trim().to_string()),
            JsonValue::from(state.channel_description.trim().to_string()),
            JsonValue::from(state.channel_generator.trim().to_string()),
            JsonValue::from(state.channel_itunes_author.trim().to_string()),
        ],
        feed_id,
    };
    write_record(&record, "newsfeeds");
}

pub fn write_nfitems(state: &ParserState, feed_id: Option<i64>) {
    let record = SqlInsert {
        table: "nfitems".to_string(),
        columns: vec![
            "feed_id".to_string(),
            "title".to_string(),
            "link".to_string(),
            "description".to_string(),
            "pub_date".to_string(),
            "itunes_image".to_string(),
            "itunes_author".to_string(),
            "podcast_funding_url".to_string(),
            "podcast_funding_text".to_string(),
        ],
        values: vec![
            match feed_id { Some(v) => JsonValue::from(v), None => JsonValue::Null },
            JsonValue::from(state.title.clone()),
            JsonValue::from(state.link.clone()),
            // Trim item description to normalize whitespace similar to channel fields
            JsonValue::from(state.description.trim().to_string()),
            JsonValue::from(state.pub_date.clone()),
            JsonValue::from(state.itunes_image.clone()),
            JsonValue::from(state.item_itunes_author.clone()),
            JsonValue::from(state.podcast_funding_url.clone()),
            JsonValue::from(state.podcast_funding_text.trim().to_string()),
        ],
        feed_id,
    };
    write_record(&record, "nfitems");
}
