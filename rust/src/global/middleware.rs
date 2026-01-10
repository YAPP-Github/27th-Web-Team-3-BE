//! Request tracing middleware for observability.
//!
//! This module provides middleware for request tracking and metrics collection.

use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use uuid::Uuid;

/// Request ID header name for distributed tracing
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Middleware that adds request tracing with unique request ID.
///
/// This middleware:
/// - Generates a unique request ID for each request
/// - Creates a tracing span with request metadata
/// - Logs request start and completion with duration
/// - Records metrics for request count and duration
pub async fn request_tracing(request: Request<Body>, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    // Create span for this request
    let span = tracing::info_span!(
        "http_request",
        request_id = %request_id,
        method = %method,
        path = %path,
    );

    let _guard = span.enter();

    tracing::info!("Request started");
    let start = Instant::now();

    // Process request
    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    tracing::info!(
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    // Record metrics
    record_request_metrics(method.as_ref(), &path, status.as_u16(), duration);

    response
}

/// Record metrics for HTTP requests
fn record_request_metrics(method: &str, path: &str, status: u16, duration: std::time::Duration) {
    let status_str = status.to_string();

    // Increment request counter
    metrics::counter!(
        "http_requests_total",
        "method" => method.to_string(),
        "path" => normalize_path(path),
        "status" => status_str.clone()
    )
    .increment(1);

    // Record response time histogram
    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method.to_string(),
        "path" => normalize_path(path),
        "status" => status_str
    )
    .record(duration.as_secs_f64());
}

/// Normalize path for metrics to avoid high cardinality
fn normalize_path(path: &str) -> String {
    // Keep only first two segments to avoid high cardinality
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() <= 2 {
        path.to_string()
    } else {
        format!("/{}/{}", segments[0], segments[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_short() {
        assert_eq!(normalize_path("/health"), "/health");
        assert_eq!(normalize_path("/api/ai"), "/api/ai");
    }

    #[test]
    fn test_normalize_path_long() {
        assert_eq!(normalize_path("/api/ai/retrospective/guide"), "/api/ai");
        assert_eq!(normalize_path("/api/ai/retrospective/refine"), "/api/ai");
    }

    #[test]
    fn test_normalize_path_root() {
        assert_eq!(normalize_path("/"), "/");
    }
}
