use axum::{extract::Request, middleware::Next, response::Response};
use tracing::{info, Instrument};
use uuid::Uuid;

#[derive(Clone)]
#[allow(dead_code)]
pub struct RequestId(pub String);

pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %method,
        uri = %path,
    );

    let request_id_for_header = request_id;
    let start = std::time::Instant::now();

    async move {
        let mut response = next.run(request).await;
        let duration_ms = start.elapsed().as_millis() as u64;
        let status = response.status().as_u16();

        info!(
            duration_ms = duration_ms,
            status = status,
            method = %method,
            path = %path,
            "request completed"
        );

        response.headers_mut().insert(
            "x-request-id",
            request_id_for_header
                .parse()
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("unknown")),
        );
        response
    }
    .instrument(span)
    .await
}
