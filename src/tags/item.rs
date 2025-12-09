use crate::outputs;
use crate::parser_state::ParserState;

pub fn on_start(state: &mut ParserState) {
    state.in_item = true;
    state.title.clear();
    state.link.clear();
    state.description.clear();
    state.pub_date.clear();
    state.itunes_image.clear();
    state.item_itunes_author.clear();
    state.podcast_funding_url.clear();
    state.podcast_funding_text.clear();
    state.in_podcast_funding = false;
    state.in_itunes_author = false;
}

pub fn on_end(feed_id: Option<i64>, state: &mut ParserState) {
    // Emit one nfitems record when closing an item
    outputs::write_nfitems(state, feed_id);
    state.in_item = false;
}
