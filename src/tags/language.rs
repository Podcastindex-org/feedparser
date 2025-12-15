use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_channel && !state.in_item && state.channel_language.is_empty() {
        state.channel_language.push_str(data);
    }
}


