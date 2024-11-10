use chrono::{NaiveDate, NaiveTime};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use soup::{NodeExt, QueryBuilderExt, Soup};

use crate::domain::NewsItem;

const NEWS_URL: &str = "https://editorial.rottentomatoes.com/news";

#[derive(Debug, Serialize, Deserialize)]
pub enum NewsItemKind {
    Article,
    Guide,
    Gallery,
}

impl NewsItemKind {
    fn try_from_url(url: impl AsRef<str>) -> anyhow::Result<Self> {
        let url = Url::parse(url.as_ref())?;
        match url.path_segments() {
            Some(mut segments) => match segments.next() {
                Some(segment) => match segment {
                    "article" => Ok(Self::Article),
                    "guide" => Ok(Self::Guide),
                    "gallery" => Ok(Self::Gallery),
                    unknown => anyhow::bail!("Unknown pattern: {}", unknown)
                },
                None => anyhow::bail!("Unknown URL format: {}", url),
            },
            None => anyhow::bail!("Unknown URL format: {}", url),
        }
    }
}

pub async fn fetch_news() -> anyhow::Result<Vec<NewsItem<NewsItemKind>>> {
    let news_html = reqwest::get(NEWS_URL)
        .await?
        .error_for_status()?
        .text()
        .await?;
    let news_soup = Soup::new(&news_html);
    let news = news_soup
        .class("articleLink")
        .find_all()
        .filter_map(|node| {
            let title = node
                .class("bannerCaption")
                .find()
                .and_then(|node| node.class("panel-body").find())
                .and_then(|node| node.class("title").find())
                .map(|node| node.text().trim().to_string());

            let date = node
                .class("bannerCaption")
                .find()
                .and_then(|node| node.class("panel-body").find())
                .and_then(|node| node.class("publication-date").find())
                .and_then(|node| NaiveDate::parse_from_str(&node.text(), "%B %d, %Y").ok())
                .map(|datetime| {
                    datetime
                        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                        .and_utc()
                        .timestamp()
                });

            let url = node.attrs().get("href").map(ToString::to_string);

            let kind = url
                .as_ref()
                .and_then(|url| NewsItemKind::try_from_url(url).ok());

            match (title, date, url, kind) {
                (Some(title), Some(date), Some(url), Some(kind)) => Some(NewsItem {
                    title,
                    date,
                    url,
                    kind,
                }),
                failure => {
                    log::warn!("Failed to parse news item: {:#?}", failure);
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    let mut unique_news: Vec<NewsItem<NewsItemKind>> = vec![];
    for item in news {
        if unique_news.iter().find(|i| i.url == item.url).is_some() {
            continue;
        }
        unique_news.push(item)
    }

    Ok(unique_news)
}
