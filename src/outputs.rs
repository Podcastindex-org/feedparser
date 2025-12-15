use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::{parser_state::ParserState, OUTPUT_SUBDIR, GLOBAL_COUNTER};
use crate::utils;

fn get_output_dir() -> PathBuf {
    OUTPUT_SUBDIR
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("outputs"))
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}


#[derive(Serialize, Deserialize, Debug)]
pub struct SqlInsert {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<JsonValue>,
    pub feed_id: Option<i64>,
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

    let title = utils::clean_string(&state.channel_title);
    let title = utils::truncate_string(&title, 768);

    let link = utils::clean_string(&state.channel_link);

    let description = if !state.channel_itunes_summary.is_empty() {
        state.channel_itunes_summary.as_str()
    } else {
        state.channel_description.as_str()
    };

    let image = if !state.channel_image.is_empty() {
        utils::sanitize_url(&state.channel_image)
    } else {
        utils::sanitize_url(&state.channel_itunes_image)
    };

    let itunes_new_feed_url = utils::sanitize_url(&state.channel_itunes_new_feed_url);
    let itunes_image = utils::sanitize_url(&state.channel_itunes_image);

    let language = utils::truncate_string(&state.channel_language, 8);
    let item_count = utils::truncate_int(state.item_count);

    let podcast_owner = if !state.channel_podcast_owner.is_empty() {
        utils::truncate_string(&state.channel_podcast_owner, 255)
    } else {
        utils::truncate_string(&state.channel_itunes_owner_email, 255)
    };

    let current_time = now_ts();
    let past_pub_dates: Vec<i64> = state.item_pubdates.clone()
        .iter()
        .filter(|&pub_date| *pub_date <= current_time)
        .map(|&pub_date| pub_date)
        .collect();

    let newest_pub_date: i64 = past_pub_dates.iter().max().copied().unwrap_or(0);
    let oldest_pub_date: i64 = past_pub_dates.iter().min().copied().unwrap_or(0);
    let update_frequency: i32 = utils::calculate_update_frequency(&past_pub_dates);

    let final_pub_date = if state.channel_pub_date != 0 {
        state.channel_pub_date
    } else if state.channel_last_build_date != 0 {
        state.channel_last_build_date
    } else {
        newest_pub_date
    };

    // get the first lightning value block, or fallback to first value if no lightning
    let podcast_value = state.channel_podcast_values
        .iter()
        .find(|value| value.model.r#type == "lightning")
        .cloned()
        .or_else(|| state.channel_podcast_values.first().cloned());

    let record = SqlInsert {
        table: "newsfeeds".to_string(),
        columns: vec![
            "feed_id".to_string(),
            "title".to_string(),
            "link".to_string(),
            "description".to_string(),
            "generator".to_string(),
            "itunes_author".to_string(),
            "type".to_string(),
            "explicit".to_string(),
            "image".to_string(),
            "language".to_string(),
            "itunes_owner_name".to_string(),
            "itunes_owner_email".to_string(),
            "atom_author_name".to_string(),
            "atom_author_email".to_string(),
            "itunes_new_feed_url".to_string(),
            "itunes_image".to_string(),
            "itunes_type".to_string(),
            "itunes_categories".to_string(),
            "podcast_guid".to_string(),
            "podcast_funding_url".to_string(),
            "podcast_funding_text".to_string(),
            "podcast_locked".to_string(),
            "podcast_value".to_string(),
            "podcast_owner".to_string(),
            "pubsub_hub_url".to_string(),
            "pubsub_self_url".to_string(),
            "pub_date".to_string(),
            "last_build_date".to_string(),
            "newest_item_pub_date".to_string(),
            "oldest_item_pub_date".to_string(),
            "item_count".to_string(),
            "update_frequency".to_string(),
        ],
        values: vec![
            match feed_id { Some(v) => JsonValue::from(v), None => JsonValue::Null },
            JsonValue::from(title),
            JsonValue::from(link),
            JsonValue::from(description),
            JsonValue::from(state.channel_generator.clone()),
            JsonValue::from(state.channel_itunes_author.clone()),
            JsonValue::from(state.feed_type.clone()),
            JsonValue::from(state.channel_explicit.clone()),
            JsonValue::from(image),
            JsonValue::from(language),
            JsonValue::from(state.channel_itunes_owner_name.clone()),
            JsonValue::from(state.channel_itunes_owner_email.clone()),
            JsonValue::from(state.channel_atom_author_name.clone()),
            JsonValue::from(state.channel_atom_author_email.clone()),
            JsonValue::from(itunes_new_feed_url),
            JsonValue::from(itunes_image),
            JsonValue::from(state.channel_itunes_type.clone()),
            JsonValue::from(state.channel_itunes_categories.clone()),
            JsonValue::from(state.channel_podcast_guid.clone()),
            JsonValue::from(state.channel_podcast_funding_url.clone()),
            JsonValue::from(state.channel_podcast_funding_text.clone()),
            JsonValue::from(state.channel_podcast_locked.clone()),
            serde_json::to_value(&podcast_value).unwrap_or(JsonValue::Null),
            JsonValue::from(podcast_owner),
            JsonValue::from(state.channel_pubsub_hub_url.clone()),
            JsonValue::from(state.channel_pubsub_self_url.clone()),
            JsonValue::from(final_pub_date),
            JsonValue::from(state.channel_last_build_date),
            JsonValue::from(newest_pub_date),
            JsonValue::from(oldest_pub_date),
            JsonValue::from(item_count),
            JsonValue::from(update_frequency),
        ],
        feed_id,
    };
    write_record(&record, "newsfeeds");
}

pub fn write_nfitems(state: &ParserState, feed_id: Option<i64>) {
    let title = utils::truncate_string(
        if !state.itunes_title.is_empty() {
            &state.itunes_title
        } else {
            &state.title.trim()
        },
        1024,
    );

    let description = if !state.content.is_empty() { // atom content
        &state.content
    } else if !state.content_encoded.is_empty() {
        &state.content_encoded
    } else if !state.description.is_empty() {
        &state.description
    } else {
        &state.itunes_summary
    }.trim();

    let link = utils::sanitize_url(&state.link);

    let guid = utils::truncate_string(
        if !state.guid.is_empty() {
            &state.guid
        } else if !state.enclosure_url.is_empty() && state.enclosure_url.len() > 10 {
            &state.enclosure_url[..state.enclosure_url.len().min(738)]
        } else {
            ""
        },
        740,
    );

    let mut enclosure_url = utils::sanitize_url(&state.enclosure_url);

    if enclosure_url.to_lowercase().contains("&amp;") {
        enclosure_url = enclosure_url.replace("&amp;", "&").to_string();
    }

    let enclosure_length = state.enclosure_length
        .parse::<i64>()
        .ok()
        .filter(|&v| v <= 922337203685477580)
        .unwrap_or(0)
        .min(922337203685477580);

    let enclosure_type = if !state.enclosure_type.is_empty() {
        utils::truncate_string(&state.enclosure_type, 128)
    } else {
        let guessed = utils::guess_enclosure_type(&state.enclosure_url);
        utils::truncate_string(&guessed, 128)
    };

    let itunes_season = state.itunes_season
        .parse::<i32>()
        .ok()
        .map(utils::truncate_int);

    let itunes_episode = if state.itunes_episode.is_empty() {
        None
    } else {
        state.itunes_episode.parse::<i32>().ok()
    };

    let image = if !state.itunes_image.is_empty() {
        utils::sanitize_url(&state.itunes_image)
    } else {
        utils::sanitize_url(&state.item_image)
    };

    // get the first lightning value block, or fallback to first value if no lightning
    let podcast_value = state.podcast_values
        .iter()
        .find(|value| value.model.r#type == "lightning")
        .cloned()
        .or_else(|| state.podcast_values.first().cloned());

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
            "guid".to_string(),
            "timestamp".to_string(),
            "enclosure_url".to_string(),
            "enclosure_length".to_string(),
            "enclosure_type".to_string(),
            "itunes_episode".to_string(),
            "itunes_episode_type".to_string(),
            "itunes_explicit".to_string(),
            "itunes_duration".to_string(),
            "image".to_string(),
            "itunes_season".to_string(),
            "podcast_transcripts".to_string(),
            "podcast_chapters".to_string(),
            "podcast_soundbites".to_string(),
            "podcast_persons".to_string(),
            "podcast_values".to_string(),
        ],
        values: vec![
            match feed_id { Some(v) => JsonValue::from(v), None => JsonValue::Null },
            JsonValue::from(title),
            JsonValue::from(link),
            JsonValue::from(description),
            JsonValue::from(state.pub_date),
            JsonValue::from(state.itunes_image.clone()),
            JsonValue::from(state.item_itunes_author.clone()),
            JsonValue::from(state.podcast_funding_url.clone()),
            JsonValue::from(state.podcast_funding_text.clone()),
            JsonValue::from(guid),
            JsonValue::from(state.pub_date),
            JsonValue::from(enclosure_url),
            JsonValue::from(enclosure_length),
            JsonValue::from(enclosure_type),
            JsonValue::from(itunes_episode),
            JsonValue::from(state.itunes_episode_type.clone()),
            JsonValue::from(state.itunes_explicit),
            JsonValue::from(state.itunes_duration),
            JsonValue::from(image),
            JsonValue::from(itunes_season),
            serde_json::to_value(&state.podcast_transcripts).unwrap_or(JsonValue::Null),
            serde_json::to_value(&state.podcast_chapters).unwrap_or(JsonValue::Null),
            serde_json::to_value(&state.podcast_soundbites).unwrap_or(JsonValue::Null),
            serde_json::to_value(&state.podcast_persons).unwrap_or(JsonValue::Null),
            serde_json::to_value(&podcast_value).unwrap_or(JsonValue::Null),
        ],
        feed_id,
    };
    write_record(&record, "nfitems");
}
