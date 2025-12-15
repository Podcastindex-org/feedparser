use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_channel_itunes_owner {
        state.channel_itunes_owner_name.clear();
        state.channel_itunes_owner_name.push_str(data);
    }
}