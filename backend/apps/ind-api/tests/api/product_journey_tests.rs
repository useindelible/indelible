use ind_test_support::spawn_app;
use reqwest::StatusCode;
use serde_json::json;

use super::common::{assert_json_response as response, assert_status};

#[tokio::test]
async fn library_journey_preserves_organization_lifecycle_and_query_contracts() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let saved = response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": "https://example.com/library-journey", "title": "Journey"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let entry = saved["library_entry_id"].as_str().unwrap();
    let document = saved["document_id"].as_str().unwrap();
    let list = response(client.get("/api/v1/library").await, StatusCode::OK).await;
    assert_eq!(list["data"][0]["library_entry_id"], entry);
    let detail = response(
        client.get(&format!("/api/v1/library/{entry}")).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(detail["document_id"], document);

    for (suffix, body, field, expected) in [
        (
            "triage",
            json!({"triage_state": "later"}),
            "triage_state",
            json!("later"),
        ),
        ("favorite", json!({}), "is_favorite", json!(true)),
        ("shortlist", json!({}), "is_shortlisted", json!(true)),
    ] {
        let changed = response(
            client
                .post_json(&format!("/api/v1/library/{entry}/{suffix}"), &body)
                .await,
            StatusCode::OK,
        )
        .await;
        assert_eq!(changed[field], expected, "{suffix}");
    }

    let tags = response(
        client
            .put_json(
                &format!("/api/v1/library/{entry}/tags"),
                &json!({"tags": ["Rust", "async"]}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(tags["tags"].as_array().unwrap().len(), 2);
    let all_tags = response(client.get("/api/v1/tags").await, StatusCode::OK).await;
    let rust_tag = all_tags["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tag| tag["name"] == "rust")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let tagged = response(
        client
            .get(&format!("/api/v1/tags/{rust_tag}/entries"))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(tagged["data"][0]["library_entry_id"], entry);

    let collection = response(
        client
            .post_json("/api/v1/collections", &json!({"name": "Reading"}))
            .await,
        StatusCode::CREATED,
    )
    .await;
    let collection = collection["id"].as_str().unwrap();
    assert_status(
        client
            .post_json(
                &format!("/api/v1/collections/{collection}/entries"),
                &json!({"library_entry_id": entry}),
            )
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;
    let members = response(
        client
            .get(&format!("/api/v1/collections/{collection}/entries"))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(members["data"][0]["document_id"], document);

    let smart_list = response(
        client
            .post_json(
                "/api/v1/smart-lists",
                &json!({
                    "name": "Later",
                    "filter_expression": {
                        "type": "condition", "field": "triage_state", "op": "eq", "value": "later"
                    }
                }),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let matches = response(
        client
            .get(&format!(
                "/api/v1/smart-lists/{}/entries",
                smart_list["id"].as_str().unwrap()
            ))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(matches["data"][0]["library_entry_id"], entry);

    assert_status(
        client.delete(&format!("/api/v1/library/{entry}")).await,
        StatusCode::NO_CONTENT,
    )
    .await;
    assert_status(
        client
            .post_json(&format!("/api/v1/library/{entry}/restore"), &json!({}))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_status(
        client
            .post_json(&format!("/api/v1/library/{entry}/purge"), &json!({}))
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;
    let resaved = response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": "https://example.com/library-journey"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(resaved["document_id"], document);
}

#[tokio::test]
async fn organization_management_crosses_tag_collection_smart_list_and_query_boundaries() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let saved = response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": "https://example.com/organization-management"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let entry = saved["library_entry_id"].as_str().unwrap();
    assert_status(
        client
            .post_json(
                &format!("/api/v1/library/{entry}/triage"),
                &json!({"triage_state": "later"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;

    let target = response(
        client
            .post_json("/api/v1/tags", &json!({"name": "Systems", "color": "blue"}))
            .await,
        StatusCode::CREATED,
    )
    .await;
    let target_id = target["id"].as_str().unwrap();
    let source = response(
        client
            .post_json("/api/v1/tags", &json!({"name": "Rust"}))
            .await,
        StatusCode::CREATED,
    )
    .await;
    let source_id = source["id"].as_str().unwrap();
    let renamed = response(
        client
            .patch_json(
                &format!("/api/v1/tags/{target_id}"),
                &json!({"name": "Systems Programming", "color": null}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(renamed["name"], "Systems Programming");
    let merged = response(
        client
            .post_json(
                "/api/v1/tags/merge",
                &json!({"source_ids": [source_id], "target_id": target_id}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(merged["id"], target_id);
    assert_eq!(
        response(
            client.get(&format!("/api/v1/tags/{target_id}")).await,
            StatusCode::OK,
        )
        .await["aliases"],
        json!(["Rust"])
    );

    let parent = response(
        client
            .post_json("/api/v1/collections", &json!({"name": "Knowledge"}))
            .await,
        StatusCode::CREATED,
    )
    .await;
    let parent_id = parent["id"].as_str().unwrap();
    let child = response(
        client
            .post_json(
                "/api/v1/collections",
                &json!({"name": "Inbox", "parent_id": parent_id}),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let child_id = child["id"].as_str().unwrap();
    let updated = response(
        client
            .patch_json(
                &format!("/api/v1/collections/{child_id}"),
                &json!({"name": "Research", "description": "Active", "sort_order": 7}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated["name"], "Research");
    let children = response(
        client
            .get(&format!("/api/v1/collections/{parent_id}/children"))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(children["data"][0]["id"], child_id);
    assert_status(
        client
            .post_json(
                &format!("/api/v1/collections/{child_id}/entries"),
                &json!({"library_entry_id": entry}),
            )
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;
    assert_status(
        client
            .delete(&format!("/api/v1/collections/{child_id}/entries/{entry}"))
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;

    let smart = response(
        client
            .post_json(
                "/api/v1/smart-lists",
                &json!({
                    "name": "Focused",
                    "filter_expression": {
                        "type": "condition", "field": "triage_state", "op": "eq", "value": "later"
                    },
                    "default_sort": "updated_at:desc"
                }),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let smart_id = smart["id"].as_str().unwrap();
    let pinned = response(
        client
            .patch_json(
                &format!("/api/v1/smart-lists/{smart_id}/pin"),
                &json!({"is_pinned": true}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(pinned["is_pinned"], true);
    let queried = response(
        client
            .post_json(
                "/api/v1/library/query",
                &json!({"filter_expression": smart["filter_expression"]}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(queried["data"][0]["library_entry_id"], entry);

    for path in [
        format!("/api/v1/smart-lists/{smart_id}"),
        format!("/api/v1/collections/{child_id}"),
        format!("/api/v1/collections/{parent_id}"),
        format!("/api/v1/tags/{target_id}"),
    ] {
        assert_status(client.delete(&path).await, StatusCode::NO_CONTENT).await;
    }
}
