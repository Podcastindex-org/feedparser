use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_item {
        state.guid.push_str(data);
    }
}


