use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_channel_itunes_owner {
        state.channel_itunes_owner_email.clear();
        state.channel_itunes_owner_email.push_str(data);
    }
}