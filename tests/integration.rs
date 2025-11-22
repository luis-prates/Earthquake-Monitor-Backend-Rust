use axum::{
    Router,
    body::{self, Body},
    http::{Request, StatusCode},
    routing::get,
};
use chrono::{TimeZone, Utc};
use earthquake_monitor_backend_rust::api_docs;
use earthquake_monitor_backend_rust::models::{Earthquake, ListResponse, Pagination};
use serde_json::Value;
use std::net::SocketAddr;
use tower::ServiceExt;
use utoipa::OpenApi;
use uuid::Uuid; // for `oneshot`

// NOTE: ensure the crate name `earthquake_monitor_backend_rust` matches your Cargo.toml `name`.
// If your crate name differs, change the `use` path above accordingly.

#[tokio::test]
async fn list_route_serializes_listresponse_ok() {
    // Build a sample Earthquake instance (same type used by the real app)
    let sample = Earthquake {
        id: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
        usgs_id: Some("usgs_test_1".to_string()),
        location: "Testville".to_string(),
        magnitude: 4.2,
        depth_km: 10.0,
        latitude: 37.0,
        longitude: -122.0,
        time: Utc.timestamp_opt(1609459200, 0).unwrap(), // 2021-01-01T00:00:00Z
    };

    let list = ListResponse {
        data: vec![sample.clone()],
        pagination: Pagination {
            limit: 50,
            offset: 0,
            total: 1,
        },
    };

    // Create a router that mounts a *mock* handler for /earthquakes
    // The mock handler simply returns the ListResponse as JSON
    let app = Router::new().route(
        "/earthquakes",
        get(move || async move {
            // return JSON body; axum will serialize
            axum::Json(list.clone())
        }),
    );

    // Build request
    let req = Request::builder()
        .uri("/earthquakes")
        .body(Body::empty())
        .unwrap();

    // Call the router in-process
    let resp = app.oneshot(req).await.expect("router oneshot failed");

    assert_eq!(resp.status(), StatusCode::OK);

    // read body bytes
    let body_bytes = body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read body");
    let body_json: Value = serde_json::from_slice(&body_bytes).expect("parse json");

    // Basic assertions on structure
    assert!(
        body_json.get("data").is_some(),
        "response should have data field"
    );
    assert!(
        body_json.get("pagination").is_some(),
        "response should have pagination field"
    );

    // assert pagination.total == 1
    assert_eq!(body_json["pagination"]["total"].as_i64().unwrap(), 1);

    // assert first item location matches
    assert_eq!(
        body_json["data"][0]["location"].as_str().unwrap(),
        "Testville"
    );
}

#[tokio::test]
async fn openapi_contains_earthquakes_path() {
    // Generate OpenAPI document from your api_docs module
    // This verifies the OpenAPI doc contains the path we expect.
    let doc = api_docs::ApiDoc::openapi();
    let value = serde_json::to_value(&doc).expect("serialize openapi");

    // Assert "paths" object has "/earthquakes" entry
    let paths = value.get("paths").expect("openapi.paths exists");
    assert!(
        paths.get("/earthquakes").is_some(),
        "openapi.paths should contain /earthquakes"
    );

    // Also ensure GET operation exists for that path (basic check)
    let get_op = &paths["/earthquakes"]["get"];
    assert!(
        get_op.is_object(),
        "GET operation should exist for /earthquakes"
    );
}
