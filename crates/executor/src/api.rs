use axum::response::IntoResponse;
use hyper::StatusCode;

pub async fn health_check_handler() -> impl IntoResponse {
    (StatusCode::OK, "Health check: OK")
}
