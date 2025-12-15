use crate::parser_state::ParserState;

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_item {
        let episode_string = data.to_string();
        let episode_digits_only: String = episode_string.chars().filter(|c| c.is_ascii_digit()).collect();

        if !episode_digits_only.is_empty() {
            if let Ok(parsed) = episode_digits_only.parse::<i64>() {
                state.itunes_episode = (parsed.min(1000000) as i32).to_string();
            }
        }
    }
}