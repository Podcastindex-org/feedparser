use crate::outputs;
use crate::parser_state::ParserState;

pub fn on_start(state: &mut ParserState) {
    state.in_channel = true;
    state.in_channel_image = false;
    state.channel_title.clear();
    state.channel_link.clear();
    state.channel_description.clear();
    state.channel_generator.clear();
    state.channel_itunes_author.clear();
}

pub fn on_end(feed_id: Option<i64>, state: &mut ParserState) {
    if state.in_channel {
        outputs::write_newsfeeds(state, feed_id);
        state.in_channel = false;
    }
}
