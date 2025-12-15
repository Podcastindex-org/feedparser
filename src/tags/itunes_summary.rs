use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_item {
        state.itunes_summary.push_str(data);
    } else if state.in_channel {
        state.channel_itunes_summary.push_str(data);
    }
}


