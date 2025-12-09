#[derive(Default)]
pub struct ParserState {
    pub in_channel: bool,
    pub in_channel_image: bool,
    pub channel_title: String,
    pub channel_link: String,
    pub channel_description: String,
    pub channel_generator: String,
    pub channel_itunes_author: String,

    pub in_item: bool,
    pub current_element: String,
    pub title: String,
    pub link: String,
    pub description: String,
    pub pub_date: String,
    pub itunes_image: String,
    pub item_itunes_author: String,
    pub podcast_funding_url: String,
    pub podcast_funding_text: String,
    pub in_podcast_funding: bool,
    pub in_itunes_author: bool,
}
