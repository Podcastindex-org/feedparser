use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_channel_atom_author {
        state.channel_atom_author_name.clear();
        state.channel_atom_author_name.push_str(data);
    }
}

