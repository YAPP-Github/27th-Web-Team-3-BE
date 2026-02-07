//! 로깅 초기화 모듈
//!
//! JSON 형식의 구조화된 로깅을 제공합니다.
//! stdout과 일별 로그 파일에 동시 출력합니다.

use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// 로깅 시스템을 초기화합니다.
///
/// JSON 포맷으로 로그를 출력하며, 환경 변수 `RUST_LOG`를 통해 로그 레벨을 설정할 수 있습니다.
/// 기본값은 `info,server=debug`입니다.
///
/// 로그는 stdout과 `logs/` 디렉토리의 일별 파일에 동시 출력됩니다.
/// 파일명 형식: `server.log.YYYY-MM-DD`
///
/// 반환되는 `WorkerGuard`를 main에서 유지해야 프로세스 종료 시 버퍼링된 로그가 손실되지 않습니다.
pub fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "logs".to_string());

    let file_appender = rolling::daily(&log_dir, "server.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let stdout_layer = fmt::layer()
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_current_span(true)
        .flatten_event(false);

    let file_layer = fmt::layer()
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_current_span(true)
        .flatten_event(false)
        .with_ansi(false)
        .with_writer(non_blocking);

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,server=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer)
        .try_init()
        .or_else(|err| {
            // Detect "already initialized" via source downcasting
            use std::error::Error;
            if err
                .source()
                .and_then(|s| s.downcast_ref::<tracing::dispatcher::SetGlobalDefaultError>())
                .is_some()
            {
                // Already initialized; this is safe to ignore
                return Ok(());
            }
            // Other initialization failures should be logged
            eprintln!("Failed to initialize tracing: {}", err);
            Err(err)
        })
        .ok(); // Convert to unit, letting the server start even if logging fails

    guard
}
