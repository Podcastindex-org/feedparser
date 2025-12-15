use crate::parser_state::ParserState;

pub fn on_start(state: &mut ParserState) {
    // Only handle Atom author at feed level (not in items)
    if state.in_channel && !state.in_item && state.feed_type == 1 {
        state.in_channel_atom_author = true;
    }
}

pub fn on_end(state: &mut ParserState) {
    state.in_channel_atom_author = false;
}

