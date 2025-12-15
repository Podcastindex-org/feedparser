use xml::attribute::OwnedAttribute;

use crate::parser_state::ParserState;

pub fn on_start(attributes: &[OwnedAttribute], state: &mut ParserState) {
    if !state.in_channel || state.in_item {
        return;
    }

    state.in_channel_podcast_locked = true;

    if let Some(attr) = attributes.iter().find(|a| a.name.local_name == "owner") {
        state.channel_podcast_owner = attr.value.clone();
    }

    if let Some(attr) = attributes.iter().find(|a| a.name.local_name == "email") {
        if state.channel_podcast_owner.trim().is_empty() {
            state.channel_podcast_owner = attr.value.clone();
        }
    }
}

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_channel_podcast_locked {
        let val = data.trim().to_ascii_lowercase();

        if matches!(val.as_str(), "yes" | "true") {
            state.channel_podcast_locked = 1;
        }
    }
}

pub fn on_end(state: &mut ParserState) {
    if state.in_channel_podcast_locked {
        state.in_channel_podcast_locked = false;
    }
}
