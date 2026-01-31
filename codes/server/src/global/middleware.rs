use axum::{extract::Request, middleware::Next, response::Response};
use tracing::Instrument;
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

    // Span에 request_id 포함 - instrument()로 async-safe하게 적용
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri().path(),
    );

    let request_id_for_header = request_id.clone();

    // instrument()를 사용하여 멀티스레드 런타임에서도 안전하게 span 유지
    async move {
        let mut response = next.run(request).await;
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
