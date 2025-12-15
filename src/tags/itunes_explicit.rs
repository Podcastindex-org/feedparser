use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    let val = data.trim().to_ascii_lowercase();
    let flag = matches!(val.as_str(), "true" | "yes" | "explicit" | "1");

    if state.in_item {
        state.itunes_explicit = if flag { 1 } else { 0 };
    } else if state.in_channel {
        state.channel_explicit = if flag { 1 } else { 0 };
    }
}


