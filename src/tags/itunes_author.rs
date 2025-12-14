use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_item {
        if state.item_itunes_author.is_empty() {
            state.item_itunes_author.push_str(data);
        }
    } else if state.in_channel {
        if state.channel_itunes_author.is_empty() {
            state.channel_itunes_author.push_str(data);
        }
    }
}