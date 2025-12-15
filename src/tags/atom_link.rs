use xml::attribute::OwnedAttribute;

use crate::parser_state::ParserState;

pub fn on_start(attributes: &[OwnedAttribute], state: &mut ParserState) {
    let mut rel = String::new();
    let mut href = String::new();
    let mut length = String::new();
    let mut link_type = String::new();

    for attr in attributes {
        match attr.name.local_name.as_str() {
            "rel" => rel = attr.value.clone(),
            "href" => href = attr.value.clone(),
            "length" => length = attr.value.clone(),
            "type" => link_type = attr.value.clone(),
            _ => {}
        }
    }

    match rel.as_str() {
        "alternate" => {
            if state.in_item && state.link.is_empty() {
                state.link = href.clone();
            } else if state.in_channel && !state.in_item && state.channel_link.is_empty() {
                state.channel_link = href.clone();
            }
        }
        "enclosure" => {
            if state.in_item && state.enclosure_url.is_empty() {
                state.enclosure_url = href;
                state.enclosure_length = length;
                state.enclosure_type = link_type;
                let url = state.enclosure_url.trim();
                if url.starts_with("http://") || url.starts_with("https://") {
                    state.item_has_valid_enclosure = true;
                }
            }
        }
        "hub" => {
            if state.in_channel && !state.in_item && state.channel_pubsub_hub_url.is_empty() {
                state.channel_pubsub_hub_url = href;
            }
        },
        "self" => {
            if state.in_channel && !state.in_item && state.channel_pubsub_self_url.is_empty() {
                state.channel_pubsub_self_url = href;
            }
        },
        _ => {}
    }
}
