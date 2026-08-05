use std::net::SocketAddr;
use std::time::Duration;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use base64::Engine;
use rand::RngCore;
use rand::rngs::OsRng;
use serde_json::json;
use tempfile::tempdir;

use investwise_server::config::Config;
use investwise_server::build_state;
use investwise_server::api::app_router;
use tower::util::ServiceExt;

#[tokio::test]
async fn relay_pair_push_poll_delete_roundtrip() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("app.db").to_string_lossy().to_string();

    // generate a 32 byte base64 secret
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    let secret_b64 = base64::engine::general_purpose::STANDARD.encode(&secret);

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

    // 1) Pair
    let pair_body = json!({"device_name": "test-device", "profile_id": "p1"}).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/relay/pair")
        .header("content-type", "application/json")
        .body(Body::from(pair_body))
        .unwrap();

    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 10_000_000).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let pairing_id = v["pairing_id"].as_str().unwrap().to_string();
    let pairing_secret = v["pairing_secret"].as_str().unwrap().to_string();

    // 2) Push
    let ciphertext_b64 = base64::engine::general_purpose::STANDARD.encode(b"hello world");
    let push_body = json!({"pairing_id": pairing_id, "seq": 1u64, "ciphertext_b64": ciphertext_b64}).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/relay/push")
        .header("content-type", "application/json")
        .header("x-pairing-secret", pairing_secret.clone())
        .body(Body::from(push_body))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3) Poll
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/relay/poll?pairing_id={}&since=0", pairing_id))
        .header("x-pairing-secret", pairing_secret.clone())
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 10_000_000).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    if let Some(items) = v.get("items").and_then(|x| x.as_array()) {
        assert!(items.len() >= 1);
    } else {
        panic!("expected items array in poll response");
    }

    // admin: list
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/relay/admin/list")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 10_000_000).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let items = v["items"].as_array().unwrap();
    assert!(items.iter().any(|it| it["pairing_id"].as_str().unwrap() == pairing_id));

    // admin: revoke
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/relay/admin/revoke?pairing_id={}", pairing_id))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // now normal delete should be NotFound
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/relay/delete?pairing_id={}", pairing_id))
        .header("x-pairing-secret", pairing_secret)
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
