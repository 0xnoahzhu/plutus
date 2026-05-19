use axum::Json;

pub async fn root() -> &'static str {
    "plutus API\n\nSee /api/v1/openapi.json for the schema, /api/v1/healthz for liveness.\n"
}

pub async fn healthz() -> &'static str {
    "ok"
}

pub async fn openapi_json() -> Json<serde_json::Value> {
    Json(crate::openapi::spec())
}
