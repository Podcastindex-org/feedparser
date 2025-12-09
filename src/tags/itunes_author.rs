use xml::attribute::OwnedAttribute;
use xml::name::OwnedName;

use crate::parser_state::ParserState;

fn is_itunes_author(name: &OwnedName) -> bool {
    name.local_name == "author"
        && (matches!(name.prefix.as_deref(), Some("itunes"))
            || matches!(
                name.namespace.as_deref(),
                Some("http://www.itunes.com/dtds/podcast-1.0.dtd")
            ))
}

pub fn on_start(name: &OwnedName, _attributes: &[OwnedAttribute], state: &mut ParserState) {
    if is_itunes_author(name) {
        state.in_itunes_author = true;
        // No other setup needed; we accumulate text in on_text and decide channel vs item there.
    }
}

pub fn on_text(_current_element: &str, data: &str, state: &mut ParserState) {
    // Accumulate only while inside a detected <itunes:author> element
    if !state.in_itunes_author {
        return;
    }

    if state.in_item {
        state.item_itunes_author.push_str(data);
    } else if state.in_channel {
        state.channel_itunes_author.push_str(data);
    }
}

pub fn on_end(name: &OwnedName, state: &mut ParserState) {
    if is_itunes_author(name) {
        state.in_itunes_author = false;
    }
}
