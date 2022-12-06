//! Clean structs for API objects, marshalled to and from JSON via serde
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct AssociatedFile {
    pub url: String,
    pub length: u64,
    pub type_str: String,
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct FeedItem {
    pub feed_id: String,
    pub item_id: String,
    pub input_id: String,
    pub title: String,
    pub content: Option<String>,
    pub canonical_url: String,
    pub item_time: String,
    pub item_time_ms: u64,
    pub deleted: bool,
    pub associated_files: Option<Vec<AssociatedFile>>,
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct InputItem {
    pub title: String,
    pub content: String,
    pub canonical_url: String,
    pub associated_files: Option<Vec<AssociatedFile>>,
}
