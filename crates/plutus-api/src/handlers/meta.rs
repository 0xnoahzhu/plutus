use axum::response::Html;
use axum::Json;

pub async fn root() -> &'static str {
    "Plutus API\n\nSee /api/v1/docs for the browseable reference, /api/v1/openapi.json for the raw spec, /api/v1/healthz for liveness.\n"
}

pub async fn healthz() -> &'static str {
    "ok"
}

pub async fn openapi_json() -> Json<serde_json::Value> {
    Json(crate::openapi::spec())
}

/// Browseable API reference. Renders Scalar (https://scalar.com) pointed at
/// our own `/api/v1/openapi.json`. The page is a single static HTML doc; all
/// rendering happens client-side from the CDN bundle, so we add no server
/// runtime cost beyond shipping the spec on demand.
pub async fn docs() -> Html<&'static str> {
    Html(SCALAR_HTML)
}

const SCALAR_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Plutus API · reference</title>
  </head>
  <body>
    <script
      id="api-reference"
      data-url="/api/v1/openapi.json"
      data-configuration='{"theme":"default","layout":"modern","hideClientButton":false}'
    ></script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
  </body>
</html>
"#;
