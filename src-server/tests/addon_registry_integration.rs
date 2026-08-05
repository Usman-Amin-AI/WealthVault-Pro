use std::net::SocketAddr;
use std::time::Duration;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::json;
use tempfile::tempdir;

use investwise_server::config::Config;
use investwise_server::build_state;
use investwise_server::api::app_router;
use base64::Engine;
use tower::util::ServiceExt;

#[tokio::test]
async fn addon_publish_list_audit() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("app.db").to_string_lossy().to_string();

    // generate a 32 byte base64 secret
    let secret_b64 = base64::engine::general_purpose::STANDARD.encode(&[0u8;32]);

    let config = Config {
        listen_addr: "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
        db_path: db_path.clone(),
        cors_allow: vec!["*".to_string()],
        request_timeout: Duration::from_millis(30000),
        static_dir: "".to_string(),
        addons_root: tmp.path().to_string_lossy().to_string(),
        secret_key: secret_b64.clone(),
        auth: None,
    };

    let state = build_state(&config).await.expect("build_state");
    let router = app_router(state.clone(), &config);

    let manifest = json!({
        "id": "test-addon",
        "name": "Test Addon",
        "version": "0.1.0",
        "description": "A test addon",
        "author": "ci",
        "files": ["index.js"]
    })
    .to_string();

    // publish
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/addons/publish")
        .header("content-type", "application/json")
        .body(Body::from(manifest.clone()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // list
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/addons")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 10_000_000).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(v["items"].as_array().unwrap().iter().any(|it| it["id"].as_str().unwrap() == "test-addon"));

    // audit
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/addons/audit")
        .header("content-type", "application/json")
        .body(Body::from(manifest))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
