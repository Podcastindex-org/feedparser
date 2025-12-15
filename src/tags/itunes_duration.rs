use crate::parser_state::ParserState;
use crate::utils;

pub fn on_start(state: &mut ParserState) {
    if state.in_item && !state.in_podcast_alternate_enclosure {
        state.itunes_duration = 0;
    }
}

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_item && !state.in_podcast_alternate_enclosure {
        state.itunes_duration = utils::time_to_seconds(data);
    }
}