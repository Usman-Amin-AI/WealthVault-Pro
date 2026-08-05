use crate::main_lib::AppState;
use crate::error::{ApiError, ApiResult};
use axum::extract::State;
use axum::routing::post;
use axum::Json;
use axum::Router;
use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine as _;
use ed25519_dalek::{PublicKey, Signature, Verifier};
use flate2::read::GzDecoder;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tar::Archive;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UploadRequest {
    pub manifest: crate::api::addon_registry::AddonManifest,
    /// package tarball (.tar.gz) base64-encoded
    pub package_b64: String,
    /// optional ed25519 signature over raw package bytes (base64)
    pub signature_b64: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/addons/upload",
    request_body = UploadRequest,
    responses((status = 200, description = "Uploaded")),
)]
#[axum::debug_handler]
pub async fn upload_addon(State(state): State<Arc<AppState>>, Json(req): Json<UploadRequest>) -> ApiResult<Json<serde_json::Value>> {
    // Validate semver
    let ver = Version::parse(&req.manifest.version).map_err(|e| ApiError::BadRequest(format!("invalid semver: {}", e)))?;

    // Decode package
    let pkg_bytes = b64.decode(&req.package_b64).map_err(|e| ApiError::BadRequest(format!("base64 decode failed: {}", e)))?;

    // Verify signature if provided
    if let Some(sig_b64) = &req.signature_b64 {
        let sig_bytes = b64.decode(sig_b64).map_err(|e| ApiError::BadRequest(format!("sig base64 decode failed: {}", e)))?;
        let pubkey_path = PathBuf::from(&state.data_root).join("addon_registry").join("pubkey.ed25519");
        if !pubkey_path.exists() {
            return Err(ApiError::BadRequest("signature provided but no server public key configured".into()));
        }
        let pubkey_b64 = fs::read_to_string(&pubkey_path).map_err(|e| ApiError::Internal(format!("reading pubkey: {}", e)))?;
        let pubkey_bytes = b64.decode(pubkey_b64.trim()).map_err(|e| ApiError::Internal(format!("pubkey decode: {}", e)))?;
        let pk = PublicKey::from_bytes(&pubkey_bytes).map_err(|e| ApiError::Internal(format!("pubkey parse: {}", e)))?;
        let signature = Signature::from_bytes(&sig_bytes).map_err(|e| ApiError::BadRequest(format!("invalid signature format: {}", e)))?;
        pk.verify(&pkg_bytes, &signature).map_err(|_| ApiError::Unauthorized)?;
    }

    // compute digest
    let mut hasher = Sha256::new();
    hasher.update(&pkg_bytes);
    let digest = hasher.finalize();

    // prepare storage path
    let id = &req.manifest.id;
    let version = ver.to_string();
    let dest = PathBuf::from(&state.data_root).join("addon_registry").join(id).join(&version);
    fs::create_dir_all(&dest).map_err(|e| ApiError::Internal(format!("mkdir: {}", e)))?;

    // write package
    let pkg_path = dest.join("package.tgz");
    fs::write(&pkg_path, &pkg_bytes).map_err(|e| ApiError::Internal(format!("write package: {}", e)))?;

    // write manifest
    let manifest_path = dest.join("manifest.json");
    let manifest_json = serde_json::to_vec_pretty(&req.manifest).map_err(|e| ApiError::Internal(format!("manifest serialize: {}", e)))?;
    fs::write(&manifest_path, &manifest_json).map_err(|e| ApiError::Internal(format!("write manifest: {}", e)))?;

    // safe extract to path tmp_extract
    let tmp_extract = dest.join("_extracted");
    if tmp_extract.exists() {
        fs::remove_dir_all(&tmp_extract).ok();
    }
    fs::create_dir_all(&tmp_extract).map_err(|e| ApiError::Internal(format!("mkdir tmp: {}", e)))?;

    // attempt gzip decode and tar extract
    let cursor = Cursor::new(&pkg_bytes);
    let gz = GzDecoder::new(cursor);
    let mut archive = Archive::new(gz);
    for entry in archive.entries().map_err(|e| ApiError::BadRequest(format!("tar entries: {}", e)))? {
        let mut entry = entry.map_err(|e| ApiError::BadRequest(format!("tar entry read: {}", e)))?;
        let path = entry.path().map_err(|e| ApiError::BadRequest(format!("entry path: {}", e)))?;
        // prevent traversal
        let clean = sanitize_path(&path);
        let outpath = tmp_extract.join(&clean);
        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent).map_err(|e| ApiError::Internal(format!("mkdir entry: {}", e)))?;
        }
        entry.unpack(&outpath).map_err(|e| ApiError::BadRequest(format!("unpack entry: {}", e)))?;
    }

    // move extracted into final 'files' dir
    let files_dir = dest.join("files");
    if files_dir.exists() {
        fs::remove_dir_all(&files_dir).ok();
    }
    fs::rename(&tmp_extract, &files_dir).map_err(|e| ApiError::Internal(format!("move extracted: {}", e)))?;

    // write metadata including digest
    let meta = serde_json::json!({
        "id": id,
        "version": version,
        "sha256": hex::encode(digest),
    });
    fs::write(dest.join("meta.json"), serde_json::to_vec_pretty(&meta).unwrap()).map_err(|e| ApiError::Internal(format!("write meta: {}", e)))?;

    Ok(Json(serde_json::json!({"status":"ok","id":id,"version":version})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/addons/upload", post(upload_addon))
}

fn sanitize_path(p: &Path) -> PathBuf {
    // remove any '..' components and leading '/'
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            std::path::Component::Normal(os) => out.push(os),
            _ => continue,
        }
    }
    out
}
