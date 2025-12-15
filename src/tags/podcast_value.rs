use xml::attribute::OwnedAttribute;

use crate::parser_state::ParserState;
use crate::models::{PodcastValue, PodcastValueModel, PodcastValueRecipient};

pub fn on_start(attributes: &[OwnedAttribute], state: &mut ParserState) {
    if !state.in_channel && !state.in_item {
        return;
    }

    let mut model_type = String::new();
    let mut model_method = String::new();
    let mut model_suggested = String::new();

    for attr in attributes {
        match attr.name.local_name.as_str() {
            "type" => model_type = attr.value.clone(),
            "method" => model_method = attr.value.clone(),
            "suggested" => model_suggested = attr.value.clone(),
            _ => {}
        }
    }

    // Check in_item first, since items are inside channels
    if state.in_item {
        state.in_podcast_value = true;
        state.value_recipients.clear();
        state.value_model_type = model_type;
        state.value_model_method = model_method;
        state.value_model_suggested = model_suggested;
    } else if state.in_channel {
        state.in_channel_podcast_value = true;
        state.channel_value_model_type = model_type;
        state.channel_value_model_method = model_method;
        state.channel_value_model_suggested = model_suggested;
    }
}

pub fn on_value_recipient(attributes: &[OwnedAttribute], state: &mut ParserState) {
    if !state.in_channel_podcast_value && !state.in_podcast_value {
        return;
    }

    let mut vr = PodcastValueRecipient::default();
    for attr in attributes {
        match attr.name.local_name.as_str() {
            "name" => vr.name = attr.value.clone(),
            "type" => vr.recipient_type = attr.value.clone(),
            "address" => vr.address = attr.value.clone(),
            "split" => vr.split = attr.value.parse().unwrap_or(0),
            "fee" => vr.fee = matches!(attr.value.to_ascii_lowercase().as_str(), "true" | "yes"),
            "customKey" => vr.custom_key = Some(attr.value.clone()),
            "customValue" => vr.custom_value = Some(attr.value.clone()),
            _ => {}
        }
    }

    // Check in_podcast_value first, since items are inside channels
    if state.in_podcast_value {
        state.value_recipients.push(vr);
    } else if state.in_channel_podcast_value {
        state.channel_value_recipients.push(vr);
    }
}

pub fn on_end(_feed_id: Option<i64>, state: &mut ParserState) {
    // Check in_podcast_value first, since items are inside channels
    if state.in_podcast_value && !state.value_recipients.is_empty() {
        state.podcast_values.push(PodcastValue {
            model: PodcastValueModel {
                r#type: state.value_model_type.clone(),
                method: state.value_model_method.clone(),
                suggested: state.value_model_suggested.clone(),
            },
            destinations: state.value_recipients.clone(),
        });
        state.in_podcast_value = false;
        state.value_recipients.clear();
        state.value_model_type.clear();
        state.value_model_method.clear();
        state.value_model_suggested.clear();
    } else if state.in_channel_podcast_value && !state.channel_value_recipients.is_empty() {
        state.channel_podcast_values.push(PodcastValue {
            model: PodcastValueModel {
                r#type: state.channel_value_model_type.clone(),
                method: state.channel_value_model_method.clone(),
                suggested: state.channel_value_model_suggested.clone(),
            },
            destinations: state.channel_value_recipients.clone(),
        });
        state.in_channel_podcast_value = false;
        state.channel_value_recipients.clear();
        state.channel_value_model_type.clear();
        state.channel_value_model_method.clear();
        state.channel_value_model_suggested.clear();
    }
}