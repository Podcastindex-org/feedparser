use xml::attribute::OwnedAttribute;

use crate::parser_state::ParserState;

pub fn on_start(attributes: &[OwnedAttribute], state: &mut ParserState) {
    let mut rel = String::new();
    let mut href = String::new();

    for attr in attributes {
        match attr.name.local_name.as_str() {
            "rel" => rel = attr.value.clone(),
            "href" => href = attr.value.clone(),
            _ => {}
        }
    }

    // Handle pubsub links (hub and self) in channel context
    if state.in_channel && !state.in_item {
        match rel.as_str() {
            "hub" => {
                if !href.is_empty() && state.channel_pubsub_hub_url.is_empty() {
                    state.channel_pubsub_hub_url = href;
                }
            }
            "self" => {
                if !href.is_empty() && state.channel_pubsub_self_url.is_empty() {
                    state.channel_pubsub_self_url = href;
                }
            }
            _ => {
                // For non-pubsub links, set channel_link if empty
                if !href.is_empty() && state.channel_link.is_empty() {
                    state.channel_link = href;
                }
            }
        }
    } else if state.in_item {
        // For items, just set the link if href is present
        if !href.is_empty() && state.link.is_empty() {
            state.link = href;
        }
    }
}

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_item {
        if state.link.is_empty() {
            state.link.push_str(data);
        }
    } else if state.in_channel && !state.in_channel_image {
        if state.channel_link.is_empty() {
            state.channel_link.push_str(data);
        }
    }
}
