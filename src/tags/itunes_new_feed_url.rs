use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_channel && !state.in_item && state.channel_itunes_new_feed_url.is_empty() {
        state.channel_itunes_new_feed_url.push_str(data);
    }
}


