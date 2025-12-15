use xml::attribute::OwnedAttribute;

use crate::parser_state::ParserState;

pub fn on_start(attributes: &[OwnedAttribute], state: &mut ParserState) {
    if !state.in_item {
        return;
    }

    // Only use the first enclosure (skip if already set)
    if !state.enclosure_url.is_empty() {
        return;
    }

    for attr in attributes {
        match attr.name.local_name.as_str() {
            "url" => state.enclosure_url = attr.value.clone(),
            "length" => state.enclosure_length = attr.value.clone(),
            "type" => state.enclosure_type = attr.value.clone(),
            _ => {}
        }
    }

    // Treat only sane URLs (http/https) as valid enclosures
    let url = state.enclosure_url.trim();
    if !url.is_empty() && (url.starts_with("http://") || url.starts_with("https://")) {
        state.item_has_valid_enclosure = true;
    }
}
