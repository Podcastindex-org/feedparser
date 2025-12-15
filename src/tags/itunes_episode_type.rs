use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_item && state.itunes_episode_type.is_empty() {
        state.itunes_episode_type.push_str(data);
    }
}