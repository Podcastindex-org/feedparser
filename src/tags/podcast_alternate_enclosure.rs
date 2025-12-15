use crate::parser_state::ParserState;

pub fn on_start(state: &mut ParserState) {
    if state.in_item {
        state.in_podcast_alternate_enclosure = true;
    }
}

pub fn on_end(state: &mut ParserState) {
    state.in_podcast_alternate_enclosure = false;
}