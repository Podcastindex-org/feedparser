use xml::attribute::OwnedAttribute;

use crate::parser_state::ParserState;

pub mod atom_author;
pub mod atom_email;
pub mod atom_link;
pub mod atom_name;
pub mod channel;
pub mod content_encoded;
pub mod description;
pub mod enclosure;
pub mod generator;
pub mod guid;
pub mod image;
pub mod item;
pub mod atom_logo;
pub mod itunes_author;
pub mod itunes_category;
pub mod itunes_email;
pub mod itunes_name;
pub mod itunes_duration;
pub mod itunes_episode;
pub mod itunes_episode_type;
pub mod itunes_explicit;
pub mod itunes_image;
pub mod itunes_new_feed_url;
pub mod itunes_owner;
pub mod itunes_season;
pub mod itunes_summary;
pub mod itunes_title;
pub mod itunes_type;
pub mod language;
pub mod last_build_date;
pub mod link;
pub mod podcast_alternate_enclosure;
pub mod podcast_chapters;
pub mod podcast_funding;
pub mod podcast_guid;
pub mod podcast_locked;
pub mod podcast_person;
pub mod podcast_soundbite;
pub mod podcast_transcript;
pub mod podcast_value;
pub mod pub_date;
pub mod content;
pub mod title;
pub mod url;

pub fn dispatch_start(current_element: &str, attributes: &[OwnedAttribute], state: &mut ParserState) {
    match current_element {
        "atom:author" | "author" => atom_author::on_start(state),
        "atom:feed" => {
            state.feed_type = 1; // atom
            channel::on_start(state);
        }
        "atom:link" => atom_link::on_start(attributes, state),
        "channel" => {
            state.feed_type = 0; // rss
            channel::on_start(state);
        }
        "enclosure" => enclosure::on_start(attributes, state),
        "image" => image::on_start(state),
        "item" | "atom:entry" => item::on_start(state),
        "itunes:category" => itunes_category::on_start(attributes, state),
        "itunes:duration" => itunes_duration::on_start(state),
        "itunes:image" => itunes_image::on_start(attributes, state),
        "itunes:owner" => itunes_owner::on_start(state),
        "link" => link::on_start( attributes, state),
        "podcast:alternateEnclosure" => podcast_alternate_enclosure::on_start(state),
        "podcast:chapters" => podcast_chapters::on_start(attributes, state),
        "podcast:funding" => podcast_funding::on_start(attributes, state),
        "podcast:locked" => podcast_locked::on_start( attributes, state),
        "podcast:person" => podcast_person::on_start(attributes, state),
        "podcast:soundbite" => podcast_soundbite::on_start(attributes, state),
        "podcast:transcript" => podcast_transcript::on_start(attributes, state),
        "podcast:value" => podcast_value::on_start(attributes, state),
        "podcast:valueRecipient" => podcast_value::on_value_recipient(attributes, state),
        _ => {}
    }
}

pub fn dispatch_text(current_element: &str, data: &str, state: &mut ParserState) {
    match current_element {
        "atom:email" | "email" => atom_email::on_text(data, state),
        "atom:logo" => atom_logo::on_text(data, state),
        "atom:name" | "name" => atom_name::on_text(data, state),
        "atom:summary" => description::on_text(data, state),
        "content" => content::on_text(data, state),
        "content:encoded" => content_encoded::on_text(data, state),
        "description" | "atom:subtitle" => description::on_text(data, state),
        "generator" => generator::on_text(data, state),
        "guid" => guid::on_text(data, state),
        "id" => guid::on_text(data, state),
        "itunes:author" => itunes_author::on_text(data, state),
        "itunes:duration" => itunes_duration::on_text(data, state),
        "itunes:email" => itunes_email::on_text(data, state),
        "itunes:episode" => itunes_episode::on_text(data, state),
        "itunes:episodeType" => itunes_episode_type::on_text(data, state),
        "itunes:explicit" => itunes_explicit::on_text(data, state),
        "itunes:image" => itunes_image::on_text(data, state),
        "itunes:name" => itunes_name::on_text(data, state),
        "itunes:new-feed-url" => itunes_new_feed_url::on_text(data, state),
        "itunes:season" => itunes_season::on_text(data, state),
        "itunes:summary" => itunes_summary::on_text(data, state),
        "itunes:title" => itunes_title::on_text(data, state),
        "itunes:type" => itunes_type::on_text(data, state),
        "language" => language::on_text(data, state),
        "lastBuildDate" => last_build_date::on_text(data, state),
        "link" => link::on_text(data, state),
        "podcast:funding" => podcast_funding::on_text(data, state),
        "podcast:guid" => podcast_guid::on_text(data, state),
        "podcast:locked" => podcast_locked::on_text(data, state),
        "podcast:person" => podcast_person::on_text(data, state),
        "podcast:soundbite" => podcast_soundbite::on_text(data, state),
        "pubDate" => pub_date::on_text(data, state),
        "published" | "atom:updated" => pub_date::on_text(data, state),
        "subtitle" => description::on_text(data, state),
        "title" => title::on_text(data, state),
        "url" => url::on_text(data, state),
        _ => {}
    }
}

pub fn dispatch_end(current_element: &str, feed_id: Option<i64>, state: &mut ParserState) {
    match current_element {
        "atom:author" | "author" => atom_author::on_end(state),
        "channel" | "atom:feed" => channel::on_end(feed_id, state),
        "image" => image::on_end(state),
        "item" | "atom:entry" => item::on_end(feed_id, state),
        "itunes:owner" => itunes_owner::on_end(state),
        "podcast:alternateEnclosure" => podcast_alternate_enclosure::on_end(state),
        "podcast:funding" => podcast_funding::on_end(state),
        "podcast:locked" | "locked" => podcast_locked::on_end(state),
        "podcast:person" => podcast_person::on_end(feed_id, state),
        "podcast:soundbite" => podcast_soundbite::on_end(feed_id, state),
        "podcast:value" => podcast_value::on_end(feed_id, state),
        _ => {}
    }
}
