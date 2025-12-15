use xml::attribute::OwnedAttribute;

use crate::parser_state::ParserState;
use crate::models::PodcastSoundbite;

pub fn on_start(attributes: &[OwnedAttribute], state: &mut ParserState) {
    if !state.in_item {
        return;
    }

    state.in_podcast_soundbite = true;
    state.current_soundbite_title.clear();
    state.current_soundbite_start.clear();
    state.current_soundbite_duration.clear();

    for attr in attributes {
        match attr.name.local_name.as_str() {
            "startTime" => state.current_soundbite_start = attr.value.clone(),
            "duration" => state.current_soundbite_duration = attr.value.clone(),
            _ => {}
        }
    }
}

pub fn on_text(data: &str, state: &mut ParserState) {
    if state.in_podcast_soundbite {
        state.current_soundbite_title.push_str(data);
    }
}

pub fn on_end(_feed_id: Option<i64>, state: &mut ParserState) {
    if state.in_podcast_soundbite {
        state.in_podcast_soundbite = false;

        use crate::utils;
        let truncated_title = utils::truncate_string(&state.current_soundbite_title, 500);

        state.podcast_soundbites.push(PodcastSoundbite {
            title: truncated_title,
            start: state.current_soundbite_start.clone(),
            duration: state.current_soundbite_duration.clone(),
        });
    }
}
