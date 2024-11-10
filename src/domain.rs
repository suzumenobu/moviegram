use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NewsItem<K> {
    pub title: String,
    pub date: i64,
    pub url: String,
    pub kind: K
}
