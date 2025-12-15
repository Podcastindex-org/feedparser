use xml::attribute::OwnedAttribute;

use crate::parser_state::ParserState;

// Detect podcast:funding start; set flag and capture optional url attribute
pub fn on_start(attributes: &[OwnedAttribute], state: &mut ParserState) {
    if state.in_item {
        state.in_podcast_funding = true;
        if let Some(attr) = attributes.iter().find(|a| a.name.local_name == "url") {
            state.podcast_funding_url = attr.value.clone();
        }
    } else if state.in_channel {
        state.in_channel_podcast_funding = true;
        if let Some(attr) = attributes.iter().find(|a| a.name.local_name == "url") {
            state.channel_podcast_funding_url = attr.value.clone();
        }
    }
}

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_podcast_funding {
        state.podcast_funding_text.push_str(data);
    } else if state.in_channel_podcast_funding {
        state.channel_podcast_funding_text.push_str(data);
    }
}

pub fn on_end(state: &mut ParserState) {
    state.in_podcast_funding = false;
    state.in_channel_podcast_funding = false;
}
