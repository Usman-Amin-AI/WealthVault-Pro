use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{extract::State, Json, Router, routing::{post, get}};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use semver::Version;

use crate::error::{ApiError, ApiResult};
use crate::main_lib::AppState;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct AddonManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub files: Option<Vec<String>>,
}

fn addon_registry_root(data_root: &str) -> PathBuf {
    let mut p = PathBuf::from(data_root);
    p.push("addon_registry");
    p
}

#[utoipa::path(
    post,
    path = "/api/v1/addons/publish",
    request_body = AddonManifest,
    responses((status = 200, description = "Published")),
)]
#[axum::debug_handler]
async fn publish_manifest(State(state): State<Arc<AppState>>, Json(man): Json<AddonManifest>) -> ApiResult<Json<serde_json::Value>> {
    // validate manifest
    if man.id.trim().is_empty() {
        return Err(ApiError::Anyhow(anyhow!("missing id")));
    }
    if Version::parse(&man.version).is_err() {
        return Err(ApiError::Anyhow(anyhow!("invalid semver version")));
    }

    let root = addon_registry_root(&state.data_root);
    fs::create_dir_all(&root).map_err(|e| ApiError::Anyhow(anyhow::anyhow!("{}", e)))?;

    let dest = root.join(&man.id).join(&man.version);
    fs::create_dir_all(&dest).map_err(|e| ApiError::Anyhow(anyhow::anyhow!("{}", e)))?;

    let manifest_path = dest.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&man).unwrap()).map_err(|e| ApiError::Anyhow(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({"status":"ok","id": man.id, "version": man.version})))
}

#[utoipa::path(
    get,
    path = "/api/v1/addons",
    responses((status = 200, description = "List")),
)]
#[axum::debug_handler]
async fn list_addons(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let root = addon_registry_root(&state.data_root);
    let mut items = vec![];
    if root.exists() {
        for entry in fs::read_dir(&root).map_err(|e| ApiError::Anyhow(anyhow::anyhow!("{}", e)))? {
            let entry = entry.map_err(|e| ApiError::Anyhow(anyhow::anyhow!("{}", e)))?;
            if entry.file_type().map_err(|e| ApiError::Anyhow(anyhow::anyhow!("{}", e)))?.is_dir() {
                let id = entry.file_name().to_string_lossy().to_string();
                let mut versions = vec![];
                if let Ok(version_entries) = fs::read_dir(entry.path()) {
                    for version_entry in version_entries.flatten() {
                        if version_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            versions.push(version_entry.file_name().to_string_lossy().to_string());
                        }
                    }
                }
                items.push(serde_json::json!({"id": id, "versions": versions}));
            }
        }
    }
    Ok(Json(serde_json::json!({"items": items})))
}

#[utoipa::path(
    post,
    path = "/api/v1/addons/audit",
    request_body = AddonManifest,
    responses((status = 200, description = "Audit result")),
)]
#[axum::debug_handler]
async fn audit_manifest(State(_state): State<Arc<AppState>>, Json(man): Json<AddonManifest>) -> ApiResult<Json<serde_json::Value>> {
    // basic security checks
    let mut issues: Vec<String> = vec![];
    if man.id.contains("..") || man.name.contains("..") {
        issues.push("invalid characters in id/name".into());
    }
    if let Some(files) = &man.files {
        for f in files {
            if f.contains("node_modules") || f.contains(".env") {
                issues.push(format!("disallowed file path: {}", f));
            }
        }
    }
    let ok = issues.is_empty();
    Ok(Json(serde_json::json!({"ok": ok, "issues": issues})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/addons/publish", post(publish_manifest))
        .route("/addons", get(list_addons))
        .route("/addons/audit", post(audit_manifest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn addon_registry_root_is_namespaced() {
        let tmp = tempdir().unwrap();
        let root = addon_registry_root(tmp.path().to_string_lossy().as_ref());
        assert!(root.ends_with("addon_registry"));
    }
}
