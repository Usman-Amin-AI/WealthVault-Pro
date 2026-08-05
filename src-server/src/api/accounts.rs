use std::sync::Arc;

use crate::{
    api::shared::{trigger_account_portfolio_job, AccountPortfolioImpact},
    error::ApiResult,
    main_lib::AppState,
    models::{Account, AccountUpdate, NewAccount},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;
use investwise_core::accounts::AccountServiceTrait;

#[derive(Deserialize, Default)]
struct ProfileQuery {
    #[serde(rename = "profile_id")]
    profile_id: Option<String>,
}

#[utoipa::path(get, path="/api/v1/accounts", responses((status=200, body = [Account])))]
async fn list_accounts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
) -> ApiResult<Json<Vec<Account>>> {
    let profile_id = query.profile_id.as_deref().unwrap_or("default_profile");
    let accounts = state.account_service.get_all_accounts(profile_id)?;
    Ok(Json(accounts.into_iter().map(Account::from).collect()))
}

#[utoipa::path(post, path="/api/v1/accounts", request_body = NewAccount, responses((status=200, body = Account)))]
async fn create_account(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
    Json(payload): Json<NewAccount>,
) -> ApiResult<Json<Account>> {
    let profile_id = query.profile_id.as_deref().unwrap_or("default_profile");
    let mut core_new: investwise_core::accounts::NewAccount = payload.into();
    core_new.profile_id = Some(profile_id.to_string());
    let created = state.account_service.create_account(core_new).await?;
    trigger_account_portfolio_job(
        state.clone(),
        AccountPortfolioImpact::CreatedOrUpdated {
            account_id: created.id.clone(),
            currency: created.currency.clone(),
        },
    );
    Ok(Json(Account::from(created)))
}

#[utoipa::path(put, path="/api/v1/accounts/{id}", request_body = AccountUpdate, responses((status=200, body=Account)))]
async fn update_account(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
    Json(mut payload): Json<AccountUpdate>,
) -> ApiResult<Json<Account>> {
    let profile_id = query.profile_id.as_deref().unwrap_or("default_profile");
    payload.id = Some(id);
    let updated = state.account_service.update_account(payload.into(), profile_id).await?;
    trigger_account_portfolio_job(
        state.clone(),
        AccountPortfolioImpact::CreatedOrUpdated {
            account_id: updated.id.clone(),
            currency: updated.currency.clone(),
        },
    );
    Ok(Json(Account::from(updated)))
}

#[utoipa::path(delete, path="/api/v1/accounts/{id}", responses((status=204)))]
async fn delete_account(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
) -> ApiResult<StatusCode> {
    let profile_id = query.profile_id.as_deref().unwrap_or("default_profile");
    state.account_service.delete_account(&id, profile_id).await?;
    trigger_account_portfolio_job(state, AccountPortfolioImpact::Deleted);
    Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/accounts", get(list_accounts).post(create_account))
        .route("/accounts/{id}", put(update_account).delete(delete_account))
}
