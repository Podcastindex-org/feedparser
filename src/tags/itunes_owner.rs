use crate::parser_state::ParserState;

pub fn on_start(state: &mut ParserState) {
    if state.in_channel && !state.in_item {
        state.in_channel_itunes_owner = true;
    }
}

pub fn on_end(state: &mut ParserState) {
    if state.in_channel_itunes_owner {
        state.in_channel_itunes_owner = false;
    }
}
