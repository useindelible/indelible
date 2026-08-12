use std::sync::Arc;

use reqwest::Client;

use super::NOTION_API_VERSION;
use super::blocks::{NotionBlock, notion_block_to_json};
use super::config::{NotionManagedTarget, NotionPropertyIds};
use super::error::NotionError;
use super::page::{NotionPageSpec, build_page_properties, spec_property_name_or_id};
use super::rate_limit::NotionRateLimiter;
use super::schema::{
    DOCUMENT_TYPE_VIEWS, extract_required_property_ids, managed_database_create_body,
};

pub struct NotionClient {
    access_token: String,
    api_base: String,
    rate_limiter: Arc<NotionRateLimiter>,
    http: Client,
}

impl NotionClient {
    pub fn new(
        access_token: String,
        api_base: String,
        rate_limiter: Arc<NotionRateLimiter>,
    ) -> Self {
        Self {
            access_token,
            api_base,
            rate_limiter,
            http: Client::new(),
        }
    }

    async fn request<B: serde::Serialize>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<serde_json::Value, NotionError> {
        self.rate_limiter.acquire().await;
        let url = format!("{}{path}", self.api_base.trim_end_matches('/'));
        let mut req = self
            .http
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Notion-Version", NOTION_API_VERSION);
        if let Some(b) = body {
            req = req.json(b);
        }
        let resp = req.send().await?;
        let status = resp.status();
        if status.as_u16() == 429 {
            let retry_after_secs = resp
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(NotionError::RateLimited { retry_after_secs });
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(NotionError::Api {
                status: status.as_u16(),
                body,
            });
        }
        Ok(resp.json::<serde_json::Value>().await?)
    }

    pub async fn find_or_create_database(&self) -> Result<NotionManagedTarget, NotionError> {
        let search_result = self
            .request(
                reqwest::Method::POST,
                "/v1/search",
                Some(&serde_json::json!({
                    "query": "Indelible",
                    "filter": {"value": "data_source", "property": "object"}
                })),
            )
            .await?;

        if let Some(first) = search_result["results"].as_array().and_then(|a| a.first())
            && let (Some(data_source_id), Some(database_id)) = (
                first["id"].as_str(),
                first["parent"]["database_id"].as_str(),
            )
        {
            let data_source = self
                .request(
                    reqwest::Method::GET,
                    &format!("/v1/data_sources/{data_source_id}"),
                    Option::<&serde_json::Value>::None,
                )
                .await?;
            if let Ok(property_ids) = extract_required_property_ids(&data_source) {
                self.ensure_document_type_views(database_id, data_source_id, &property_ids)
                    .await?;
                return Ok(NotionManagedTarget {
                    database_id: database_id.to_string(),
                    data_source_id: data_source_id.to_string(),
                    property_ids,
                });
            }
        }

        let create_body = managed_database_create_body();
        let create_result = self
            .request(reqwest::Method::POST, "/v1/databases", Some(&create_body))
            .await?;

        let database_id = create_result["id"]
            .as_str()
            .ok_or_else(|| NotionError::Api {
                status: 0,
                body: "missing id in create response".into(),
            })
            .map(|s| s.to_string())?;

        let data_source_id = if let Some(id) = create_result["data_sources"]
            .as_array()
            .and_then(|sources| sources.first())
            .and_then(|source| source["id"].as_str())
        {
            id.to_string()
        } else {
            let retrieved = self
                .request(
                    reqwest::Method::GET,
                    &format!("/v1/databases/{database_id}"),
                    Option::<&serde_json::Value>::None,
                )
                .await?;
            retrieved["data_sources"]
                .as_array()
                .and_then(|sources| sources.first())
                .and_then(|source| source["id"].as_str())
                .ok_or_else(|| {
                    NotionError::State("missing data source id after database create".into())
                })?
                .to_string()
        };

        let data_source = self
            .request(
                reqwest::Method::GET,
                &format!("/v1/data_sources/{data_source_id}"),
                Option::<&serde_json::Value>::None,
            )
            .await?;
        let property_ids = extract_required_property_ids(&data_source)?;
        self.ensure_document_type_views(&database_id, &data_source_id, &property_ids)
            .await?;

        Ok(NotionManagedTarget {
            database_id,
            data_source_id,
            property_ids,
        })
    }

    pub async fn validate_managed_target(
        &self,
        database_id: &str,
        data_source_id: &str,
    ) -> Result<NotionManagedTarget, NotionError> {
        let data_source = self
            .request(
                reqwest::Method::GET,
                &format!("/v1/data_sources/{data_source_id}"),
                Option::<&serde_json::Value>::None,
            )
            .await?;
        let property_ids = extract_required_property_ids(&data_source)?;
        Ok(NotionManagedTarget {
            database_id: database_id.to_string(),
            data_source_id: data_source_id.to_string(),
            property_ids,
        })
    }

    async fn ensure_document_type_views(
        &self,
        database_id: &str,
        data_source_id: &str,
        property_ids: &NotionPropertyIds,
    ) -> Result<(), NotionError> {
        // List existing views before deciding what to create. A failure here
        // (auth expired, network issue, Notion 5xx) means we cannot tell
        // which views already exist — proceeding would either skip every
        // view (if we treated the failure as "nothing here") or duplicate
        // every view (if we treated it as "no matches"). Bubble the error.
        let existing = self
            .request(
                reqwest::Method::GET,
                &format!("/v1/views?database_id={database_id}&page_size=100"),
                Option::<&serde_json::Value>::None,
            )
            .await?;

        let mut names = std::collections::HashSet::new();
        if let Some(results) = existing["results"].as_array() {
            for result in results {
                if let Some(name) = result["view"]["name"]
                    .as_str()
                    .or_else(|| result["name"].as_str())
                {
                    names.insert(name.to_string());
                }
            }
        }

        for &(name, category) in DOCUMENT_TYPE_VIEWS {
            if names.contains(name) {
                continue;
            }
            self.request(
                reqwest::Method::POST,
                "/v1/views",
                Some(&serde_json::json!({
                    "database_id": database_id,
                    "data_source_id": data_source_id,
                    "name": name,
                    "type": "table",
                    "filter": {
                        "property": property_ids.category,
                        "select": {"equals": category}
                    }
                })),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn upsert_page(
        &self,
        data_source_id: &str,
        known_page_id: Option<&str>,
        spec: &NotionPageSpec,
    ) -> Result<String, NotionError> {
        let existing_page_id = if let Some(page_id) = known_page_id {
            Some(page_id.to_string())
        } else {
            let query_result = self
                .request(
                    reqwest::Method::POST,
                    &format!("/v1/data_sources/{data_source_id}/query"),
                    Some(&serde_json::json!({
                        "filter": {
                            "property": spec_property_name_or_id(&spec.property_ids.indelible_id, "Indelible ID"),
                            "rich_text": {"equals": spec.indelible_id}
                        }
                    })),
                )
                .await?;

            query_result["results"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|p| p["id"].as_str())
                .map(|s| s.to_string())
        };

        let page_properties = build_page_properties(spec);

        if let Some(page_id) = existing_page_id {
            self.request(
                reqwest::Method::PATCH,
                &format!("/v1/pages/{page_id}"),
                Some(&serde_json::json!({"properties": page_properties})),
            )
            .await?;
            Ok(page_id)
        } else {
            let result = self
                .request(
                    reqwest::Method::POST,
                    "/v1/pages",
                    Some(&serde_json::json!({
                        "parent": {"type": "data_source_id", "data_source_id": data_source_id},
                        "properties": page_properties
                    })),
                )
                .await?;
            result["id"]
                .as_str()
                .ok_or_else(|| NotionError::Api {
                    status: 0,
                    body: "missing id in page create response".into(),
                })
                .map(|s| s.to_string())
        }
    }

    pub async fn append_blocks(
        &self,
        page_id: &str,
        blocks: &[NotionBlock],
    ) -> Result<(), NotionError> {
        let blocks_json: Vec<serde_json::Value> = blocks.iter().map(notion_block_to_json).collect();
        self.request(
            reqwest::Method::PATCH,
            &format!("/v1/blocks/{page_id}/children"),
            Some(&serde_json::json!({"children": blocks_json})),
        )
        .await?;
        Ok(())
    }

    pub async fn archive_page(&self, page_id: &str) -> Result<String, NotionError> {
        let page = self
            .request(
                reqwest::Method::PATCH,
                &format!("/v1/pages/{page_id}"),
                Some(&serde_json::json!({"in_trash": true})),
            )
            .await?;
        page["url"].as_str().map(str::to_string).ok_or_else(|| {
            NotionError::State("archived page response did not include its URL".into())
        })
    }
}
