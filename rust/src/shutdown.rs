use tokio::signal;

/// Graceful shutdown을 위한 시그널 핸들러
///
/// SIGTERM 또는 SIGINT(Ctrl+C) 시그널을 수신하면 반환합니다.
/// 이를 통해 서버가 진행 중인 요청을 완료한 후 종료할 수 있습니다.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM signal");
        },
    }

    tracing::info!("Initiating graceful shutdown...");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn shutdown_signal_should_not_complete_immediately() {
        // shutdown_signal은 시그널을 받을 때까지 대기해야 함
        // 짧은 타임아웃으로 테스트하여 즉시 완료되지 않는지 확인
        let result = timeout(Duration::from_millis(10), shutdown_signal()).await;

        // 타임아웃 발생 = 시그널 대기 중 (정상)
        assert!(result.is_err(), "shutdown_signal should wait for signal");
    }
}
