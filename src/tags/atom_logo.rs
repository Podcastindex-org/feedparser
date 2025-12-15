use crate::parser_state::ParserState;

/// Handle Atom `<logo>` values as channel-level images.
pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_channel && !state.in_item && state.channel_image.is_empty() {
        state.channel_image.push_str(data);
    }
}


