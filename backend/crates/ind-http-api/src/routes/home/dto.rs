use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::FieldError;
use crate::extract::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct HomeDashboardResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_reading: Option<ContinueReadingWidget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_review: Option<DailyReviewWidget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recently_added: Option<RecentlyAddedWidget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quick_reads: Option<QuickReadsWidget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_collections: Option<PinnedCollectionsWidget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading_stats: Option<ReadingStatsWidget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_digest: Option<FeedDigestWidget>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ContinueReadingWidget {
    pub items: Vec<HomeItemResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DailyReviewWidget {
    pub due_count: i64,
    pub streak_days: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RecentlyAddedWidget {
    pub items: Vec<HomeItemResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QuickReadsWidget {
    pub items: Vec<HomeItemResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PinnedCollectionsWidget {
    pub collections: Vec<PinnedCollectionEntry>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PinnedCollectionEntry {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub recent_items: Vec<HomeItemResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReadingStatsWidget {
    pub documents_read: i64,
    pub items_completed: i64,
    pub highlights_made: i64,
    pub streak_days: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FeedDigestWidget {
    pub items: Vec<HomeFeedItemResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HomeItemResponse {
    pub id: String,
    pub item_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading_time_minutes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_progress_percent: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_read_at: Option<DateTime<Utc>>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl HomeItemResponse {
    pub fn from_domain(document: ind_application::repos::home::HomeDocument) -> Self {
        Self {
            id: document.document_id.to_string(),
            item_type: serde_json::to_value(document.item_type)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default(),
            title: document.title,
            excerpt: document.excerpt,
            url: document.url,
            domain: document.domain,
            author: document.author,
            reading_time_minutes: document.reading_time_minutes,
            thumbnail_url: document.lead_image_url.clone(),
            lead_image_url: document.lead_image_url,
            progress_percent: document.progress_percent,
            max_progress_percent: document.max_progress_percent,
            last_read_at: document.last_read_at,
            created_at: document.created_at,
        }
    }

    pub fn enrich_thumbnail(&mut self, resolved: &std::collections::HashMap<String, String>) {
        if let Some(url) = resolved.get(&self.id) {
            self.thumbnail_url = Some(url.clone());
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HomeFeedItemResponse {
    pub id: String,
    pub subscription_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub published_at: Option<DateTime<Utc>>,
}

impl HomeFeedItemResponse {
    pub fn from_domain(entry: ind_application::repos::home::HomeFeedEntry) -> Self {
        Self {
            id: entry.delivery_id.to_string(),
            subscription_id: entry.subscription_id.to_string(),
            title: entry.title,
            url: entry.url,
            author: entry.author,
            excerpt: entry.excerpt,
            published_at: entry.published_at,
        }
    }
}

impl HomeDashboardResponse {
    pub fn enrich_thumbnails(&mut self, resolved: &std::collections::HashMap<String, String>) {
        let enrich = |items: &mut Vec<HomeItemResponse>| {
            for item in items.iter_mut() {
                item.enrich_thumbnail(resolved);
            }
        };
        if let Some(w) = self.continue_reading.as_mut() {
            enrich(&mut w.items);
        }
        if let Some(w) = self.recently_added.as_mut() {
            enrich(&mut w.items);
        }
        if let Some(w) = self.quick_reads.as_mut() {
            enrich(&mut w.items);
        }
        if let Some(w) = self.pinned_collections.as_mut() {
            for col in w.collections.iter_mut() {
                enrich(&mut col.recent_items);
            }
        }
    }

    pub fn from_dashboard(data: ind_application::HomeDashboardData) -> Self {
        Self {
            continue_reading: data.continue_reading.map(|items| ContinueReadingWidget {
                items: items
                    .into_iter()
                    .map(HomeItemResponse::from_domain)
                    .collect(),
            }),
            daily_review: data.daily_review.map(|dr| DailyReviewWidget {
                due_count: dr.due_count,
                streak_days: dr.streak_days,
            }),
            recently_added: data.recently_added.map(|items| RecentlyAddedWidget {
                items: items
                    .into_iter()
                    .map(HomeItemResponse::from_domain)
                    .collect(),
            }),
            quick_reads: data.quick_reads.map(|items| QuickReadsWidget {
                items: items
                    .into_iter()
                    .map(HomeItemResponse::from_domain)
                    .collect(),
            }),
            pinned_collections: data.pinned_collections.map(|cols| PinnedCollectionsWidget {
                collections: cols
                    .into_iter()
                    .map(|(col, items)| PinnedCollectionEntry {
                        id: col.id.to_string(),
                        name: col.name,
                        description: col.description,
                        icon: col.icon,
                        color: col.color,
                        recent_items: items
                            .into_iter()
                            .map(HomeItemResponse::from_domain)
                            .collect(),
                    })
                    .collect(),
            }),
            reading_stats: data.reading_stats.map(|stats| ReadingStatsWidget {
                documents_read: stats.documents_read,
                items_completed: stats.items_completed,
                highlights_made: stats.highlights_made,
                streak_days: stats.streak_days,
            }),
            feed_digest: data.feed_digest.map(|items| FeedDigestWidget {
                items: items
                    .into_iter()
                    .map(HomeFeedItemResponse::from_domain)
                    .collect(),
            }),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HomeSettingsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<HomeWidgetConfig>)]
    pub widget_config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct HomeWidgetConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget_order: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_widgets: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateHomeSettingsBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget_order: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_widgets: Option<Vec<String>>,
}

impl Validate for UpdateHomeSettingsBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let valid: Vec<String> = ind_domain::HomeWidgetKind::ALL
            .iter()
            .filter_map(|k| {
                serde_json::to_value(k)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
            })
            .collect();

        let mut errors = Vec::new();

        if let Some(order) = &self.widget_order {
            for (i, name) in order.iter().enumerate() {
                if !valid.contains(name) {
                    errors.push(FieldError {
                        field: format!("widget_order[{i}]"),
                        message: format!("unknown widget '{name}'"),
                    });
                }
            }
        }

        if let Some(hidden) = &self.hidden_widgets {
            for (i, name) in hidden.iter().enumerate() {
                if !valid.contains(name) {
                    errors.push(FieldError {
                        field: format!("hidden_widgets[{i}]"),
                        message: format!("unknown widget '{name}'"),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct HomeDashboardParams {
    pub widgets: Option<String>,
}
