use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_item && state.content.is_empty() {
        state.content.push_str(data);
    }
}