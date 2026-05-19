use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use plutus_core::cost_basis::CostBasisMethod;

use crate::dto::holding::HoldingOut;
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct HoldingsFilter {
    pub account_id: Option<i64>,
    pub method: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(f): Query<HoldingsFilter>,
) -> ApiResult<Json<Vec<HoldingOut>>> {
    let method = match f.method.as_deref().unwrap_or("fifo") {
        "fifo" => CostBasisMethod::Fifo,
        "lifo" => CostBasisMethod::Lifo,
        "average" => CostBasisMethod::Average,
        other => {
            return Err(ApiError::BadRequest(format!(
                "method must be fifo/lifo/average; got {other}"
            )))
        }
    };
    let rows = if let Some(account_id) = f.account_id {
        plutus_storage::queries::holdings::compute_for_account(&state.db, account_id, method)
            .await?
    } else {
        plutus_storage::queries::holdings::compute_all(&state.db, method).await?
    };
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}
