use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    let text = data.trim();

    if state.in_channel_image {
        state.channel_image.push_str(text);
    } else if state.in_item_image {
        state.item_image.push_str(text);
    }
}


