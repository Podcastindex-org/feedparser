use xml::attribute::OwnedAttribute;
use crate::parser_state::ParserState;

pub fn on_start(attributes: &[OwnedAttribute], state: &mut ParserState) {
    if state.in_channel && !state.in_item {
        if let Some(attr) = attributes.iter().find(|a| a.name.local_name == "text") {
            let val = attr.value.trim();
            if !val.is_empty() {
                state.channel_itunes_categories.push(val.to_string());
            }
        }
    }
}