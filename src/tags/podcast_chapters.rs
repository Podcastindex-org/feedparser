use xml::attribute::OwnedAttribute;

use crate::parser_state::ParserState;
use crate::models::PodcastChapter;

pub fn on_start(attributes: &[OwnedAttribute], state: &mut ParserState) {
    if !state.in_item || state.in_podcast_alternate_enclosure {
        return;
    }


    let mut chapter_url = String::new();
    let mut chapter_type = String::new();

    for attr in attributes {
        match attr.name.local_name.as_str() {
            "url" => chapter_url = attr.value.clone(),
            "type" => chapter_type = attr.value.clone(),
            _ => {}
        }
    }

    state.podcast_chapters.push(PodcastChapter {
        url: chapter_url,
        r#type: chapter_type,
    });
}

