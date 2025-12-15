use serde::Serialize;

#[derive(Serialize)]
pub struct PodcastTranscript {
    pub url: String,
    pub r#type: String,
}

#[derive(Serialize)]
pub struct PodcastChapter {
    pub url: String,
    pub r#type: String,
}

#[derive(Serialize)]
pub struct PodcastSoundbite {
    pub title: String,
    pub start: String,
    pub duration: String,
}

#[derive(Serialize)]
pub struct PodcastPerson {
    pub name: String,
    pub role: String,
    pub group: String,
    pub img: String,
    pub href: String,
}

#[derive(Serialize, Clone)]
pub struct PodcastValue {
    pub model: PodcastValueModel,
    pub destinations: Vec<PodcastValueRecipient>,
}

#[derive(Serialize, Clone)]
pub struct PodcastValueModel {
    pub r#type: String,
    pub method: String,
    pub suggested: String,
}

#[derive(Serialize, Clone, Default)]
pub struct PodcastValueRecipient {
    pub name: String,
    pub recipient_type: String,
    pub address: String,
    pub split: i32,
    pub fee: bool,
    pub custom_key: Option<String>,
    pub custom_value: Option<String>,
}