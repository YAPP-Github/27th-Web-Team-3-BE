//! 로깅 초기화 모듈
//!
//! JSON 형식의 구조화된 로깅을 제공합니다.

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// 로깅 시스템을 초기화합니다.
///
/// JSON 포맷으로 로그를 출력하며, 환경 변수 `RUST_LOG`를 통해 로그 레벨을 설정할 수 있습니다.
/// 기본값은 `info,server=debug`입니다.
pub fn init_logging() {
    // flatten_event(false)로 설정하여 fields 중첩 구조 유지
    let fmt_layer = fmt::layer()
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_current_span(true)
        .flatten_event(false);

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,server=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
