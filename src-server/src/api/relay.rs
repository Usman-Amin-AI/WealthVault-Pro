use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    routing::{delete, get, post},
    Json, Router,
};
use anyhow::anyhow;
use base64::Engine;
// chrono imports
use rand::RngCore;
use rand::rngs::OsRng;
use argon2::{Argon2, password_hash::{SaltString, rand_core::OsRng as PHOsRng, PasswordHash, PasswordHasher, PasswordVerifier}};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::shared::normalize_profile_id;
use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, ToSchema)]
struct CreatePairBody {
    #[serde(default)]
    device_name: Option<String>,
    #[serde(default)]
    profile_id: Option<String>,
}

#[derive(Serialize, ToSchema)]
struct PairResponse {
    pairing_id: String,
    pairing_secret: String,
}

#[derive(Deserialize, ToSchema)]
struct PushBody {
    pairing_id: String,
    seq: u64,
    ciphertext_b64: String,
}

#[derive(Deserialize, ToSchema)]
struct PollQuery {
    pairing_id: String,
    #[serde(default)]
    since: Option<u64>,
}

fn relays_root(data_root: &str) -> PathBuf {
    let mut p = PathBuf::from(data_root);
    p.push("relays");
    p
}

fn pairing_dir(data_root: &str, pairing_id: &str) -> PathBuf {
    let mut p = relays_root(data_root);
    p.push(pairing_id);
    p
}

#[utoipa::path(
    post,
    path = "/api/v1/relay/pair",
    request_body = CreatePairBody,
    responses((status = 200, description = "Pair created", body = PairResponse)),
)]
async fn create_pair(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreatePairBody>,
) -> ApiResult<Json<PairResponse>> {
    let pairing_id = Uuid::new_v4().to_string();
    let mut secret = [0u8; 32];
    let mut rng = OsRng;
    rng.fill_bytes(&mut secret);
    let pairing_secret = base64::engine::general_purpose::STANDARD.encode(&secret);

    // Hash the secret with Argon2 and store the hash (not the raw secret)
    let salt = SaltString::generate(&mut PHOsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(pairing_secret.as_bytes(), &salt)
        .map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?
        .to_string();

    let dir = pairing_dir(&state.data_root, &pairing_id);
    fs::create_dir_all(&dir).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;

    let meta = serde_json::json!({
        "device_name": body.device_name.unwrap_or_default(),
        "profile_id": normalize_profile_id(body.profile_id.as_deref()),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "last_seq": 0u64,
        "pairing_secret_hash": password_hash,
    });
    let mut f = fs::File::create(dir.join("meta.json")).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;
    f.write_all(serde_json::to_string_pretty(&meta).unwrap().as_bytes())
        .map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;

    Ok(Json(PairResponse { pairing_id, pairing_secret }))
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/api/v1/relay/push",
    request_body = PushBody,
    responses((status = 200, description = "Blob stored")),
)]
async fn push_blob(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<PushBody>,
) -> ApiResult<Json<String>> {
    let dir = pairing_dir(&state.data_root, &body.pairing_id);
    if !dir.exists() {
        return Err(ApiError::NotFound);
    }

    validate_pairing_secret(&dir, &headers)?;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&body.ciphertext_b64)
        .map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;

    let file_path = dir.join(format!("blob_{:020}.bin", body.seq));
    let mut f = fs::File::create(&file_path).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;
    f.write_all(&bytes).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;

    // update meta last_seq
    let meta_path = dir.join("meta.json");
    if meta_path.exists() {
        if let Ok(meta_str) = fs::read_to_string(&meta_path) {
            if let Ok(mut meta_val) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                meta_val["last_seq"] = serde_json::Value::from(body.seq);
                let mut f = fs::File::create(&meta_path).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;
                f.write_all(serde_json::to_string_pretty(&meta_val).unwrap().as_bytes())
                    .map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;
            }
        }
    }

    Ok(Json("ok".to_string()))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/api/v1/relay/poll",
    responses((status = 200, description = "List blobs")),
)]
async fn poll_blobs(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PollQuery>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    let dir = pairing_dir(&state.data_root, &q.pairing_id);
    if !dir.exists() {
        return Err(ApiError::NotFound);
    }

    validate_pairing_secret(&dir, &headers)?;

    let since = q.since.unwrap_or(0);
    let mut blobs = vec![];
    for entry in fs::read_dir(&dir).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))? {
        let entry = entry.map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("blob_") && name.ends_with(".bin") {
            if let Some(seq_str) = name.strip_prefix("blob_") {
                if let Some(seq_str) = seq_str.strip_suffix(".bin") {
                    if let Ok(seq) = seq_str.parse::<u64>() {
                        if seq > since {
                            let data = fs::read(entry.path()).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;
                            let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                            blobs.push(serde_json::json!({"seq": seq, "ciphertext_b64": b64}));
                        }
                    }
                }
            }
        }
    }

    blobs.sort_by_key(|v| v["seq"].as_u64().unwrap_or(0));
    Ok(Json(serde_json::json!({"items": blobs})))
}

#[axum::debug_handler]
#[utoipa::path(
    delete,
    path = "/api/v1/relay/delete",
    responses((status = 200, description = "Deleted")),
)]
async fn delete_pair(
    State(state): State<Arc<AppState>>,
    Query(q): Query<std::collections::HashMap<String, String>>,
    headers: HeaderMap,
) -> ApiResult<Json<String>> {
    if let Some(pairing_id) = q.get("pairing_id") {
        let dir = pairing_dir(&state.data_root, pairing_id);
        if dir.exists() {
            validate_pairing_secret(&dir, &headers)?;
            fs::remove_dir_all(&dir).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;
            return Ok(Json("deleted".to_string()));
        }
    }
    Err(ApiError::NotFound)
}

fn validate_pairing_secret(dir: &PathBuf, headers: &HeaderMap) -> Result<(), ApiError> {
    let meta_path = dir.join("meta.json");
    if !meta_path.exists() {
        return Err(ApiError::NotFound);
    }
    let meta_str = fs::read_to_string(&meta_path).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;
    let meta_val: serde_json::Value = serde_json::from_str(&meta_str).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;
    let expected_hash = meta_val
        .get("pairing_secret_hash")
        .and_then(|v| v.as_str())
        .ok_or(ApiError::Unauthorized)?;

    if let Some(v) = headers.get("x-pairing-secret") {
        let provided = v.to_str().map_err(|_| ApiError::Unauthorized)?;
        let parsed = PasswordHash::new(expected_hash).map_err(|_| ApiError::Unauthorized)?;
        let argon2 = Argon2::default();
        argon2
            .verify_password(provided.as_bytes(), &parsed)
            .map_err(|_| ApiError::Unauthorized)?;
        return Ok(());
    }
    Err(ApiError::Unauthorized)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/relay/pair", post(create_pair))
        .route("/relay/push", post(push_blob))
        .route("/relay/poll", get(poll_blobs))
        .route("/relay/delete", delete(delete_pair))
        .route("/relay/admin/list", get(admin_list))
        .route("/relay/admin/revoke", delete(admin_revoke))
        .route("/relay/admin/prune", post(admin_prune))
}

async fn admin_list(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let root = relays_root(&state.data_root);
    let mut items = vec![];
    if root.exists() {
        for entry in fs::read_dir(&root).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))? {
            let entry = entry.map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;
            if entry.file_type().map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?.is_dir() {
                let meta_path = entry.path().join("meta.json");
                if meta_path.exists() {
                    if let Ok(meta_str) = fs::read_to_string(&meta_path) {
                        if let Ok(mut meta_val) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                            meta_val["pairing_id"] = serde_json::Value::from(entry.file_name().to_string_lossy().to_string());
                            items.push(meta_val);
                        }
                    }
                }
            }
        }
    }
    Ok(Json(serde_json::json!({"items": items})))
}

async fn admin_revoke(
    State(state): State<Arc<AppState>>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<String>> {
    if let Some(pairing_id) = q.get("pairing_id") {
        let dir = pairing_dir(&state.data_root, pairing_id);
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;
            return Ok(Json("revoked".to_string()));
        }
    }
    Err(ApiError::NotFound)
}

fn prune_pairings(data_root: &str, retention: Duration) -> Result<u64, anyhow::Error> {
    let root = relays_root(data_root);
    let mut removed = 0u64;
    if !root.exists() {
        return Ok(0);
    }
    let now = SystemTime::now();
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let meta_path = entry.path().join("meta.json");
        if !meta_path.exists() {
            continue;
        }
        let meta_str = fs::read_to_string(&meta_path)?;
        let meta_val: serde_json::Value = serde_json::from_str(&meta_str)?;
        if let Some(created_at) = meta_val.get("created_at").and_then(|v| v.as_str()) {
            if let Ok(dt) = DateTime::parse_from_rfc3339(created_at) {
                let created_system = SystemTime::from(dt.with_timezone(&Utc));
                if now.duration_since(created_system).unwrap_or_default() > retention {
                    fs::remove_dir_all(entry.path())?;
                    removed += 1;
                }
            }
        }
    }
    Ok(removed)
}

async fn admin_prune(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let default_secs = 60 * 60 * 24 * 30; // 30 days
    let secs = std::env::var("WF_RELAY_RETENTION_SECS").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(default_secs);
    let removed = prune_pairings(&state.data_root, Duration::from_secs(secs)).map_err(|e| ApiError::Anyhow(anyhow!("{}", e)))?;
    Ok(Json(serde_json::json!({"removed": removed})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn helper_paths_work() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_string_lossy().to_string();
        let pid = "abc-123";
        let dir = pairing_dir(&root, pid);
        assert!(dir.to_string_lossy().contains("relays"));
    }
}
