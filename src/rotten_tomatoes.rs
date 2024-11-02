use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use soup::{NodeExt, QueryBuilderExt, Soup};

use crate::domain::NewsItem;

const NEWS_URL: &'static str = "https://editorial.rottentomatoes.com/news";

pub async fn fetch_news() -> anyhow::Result<Vec<NewsItem>> {
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

            match (title, date) {
                (Some(title), Some(date)) => Some(NewsItem {
                    title,
                    date,
                    url: node.attrs().get("href").unwrap().to_string(),
                }),
                _ => {
                    log::warn!("Failed to parse title or date");
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    Ok(news)
}
