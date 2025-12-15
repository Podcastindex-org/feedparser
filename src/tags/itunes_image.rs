use xml::attribute::OwnedAttribute;

use crate::parser_state::ParserState;

// Handle both channel- and item-level itunes:image elements, supporting the
// common href/url attribute as well as text content fallback.
pub fn on_start(attributes: &[OwnedAttribute], state: &mut ParserState) {
    if let Some(attr) = attributes.iter().find(|a| {
        let key = a.name.local_name.as_str();
        key == "href" || key == "url"
    }) {
        if state.in_item {
            state.itunes_image = attr.value.clone();
        } else if state.in_channel {
            state.channel_itunes_image = attr.value.clone();
        }
    }
}

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_item {
        state.itunes_image.push_str(data);
    } else if state.in_channel {
        state.channel_itunes_image.push_str(data);
    }
}