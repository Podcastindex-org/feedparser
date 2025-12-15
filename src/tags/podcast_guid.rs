use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_channel && !state.in_item {
        state.channel_podcast_guid.push_str(data);
    }
}


