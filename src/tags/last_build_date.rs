use crate::parser_state::ParserState;
use crate::utils;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_channel && !state.in_item && state.channel_last_build_date == 0 {
        state.channel_last_build_date = utils::pub_date_to_timestamp(data);
    }
}

