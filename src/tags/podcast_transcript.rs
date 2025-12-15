use xml::attribute::OwnedAttribute;

use crate::parser_state::ParserState;
use crate::models::PodcastTranscript;

pub fn on_start(attributes: &[OwnedAttribute], state: &mut ParserState) {
    if !state.in_item || state.in_podcast_alternate_enclosure {
        return;
    }

    let mut transcript_url = String::new();
    let mut transcript_type = String::new();

    for attr in attributes {
        match attr.name.local_name.as_str() {
            "url" => transcript_url = attr.value.clone(),
            "type" => transcript_type = attr.value.clone(),
            _ => {}
        }
    }

    state.podcast_transcripts.push(PodcastTranscript {
        url: transcript_url,
        r#type: transcript_type,
    });
}