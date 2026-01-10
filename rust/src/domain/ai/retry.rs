use backoff::{future::retry, ExponentialBackoff};
use std::future::Future;
use std::time::Duration;

use crate::error::AppError;

/// 재시도 가능한 에러인지 판단
///
/// Rate limit, timeout, 서버 에러(5xx) 등 일시적 오류는 재시도
/// 인증 오류, 요청 형식 오류 등 영구적 오류는 즉시 실패
fn is_retryable_error(error: &AppError) -> bool {
    match error {
        AppError::OpenAiError(msg) => {
            let msg_lower = msg.to_lowercase();
            // Rate limit, timeout, 5xx 서버 에러
            msg_lower.contains("rate limit")
                || msg_lower.contains("timeout")
                || msg_lower.contains("timed out")
                || msg_lower.contains("429") // Too Many Requests
                || msg_lower.contains("500")
                || msg_lower.contains("502")
                || msg_lower.contains("503")
                || msg_lower.contains("504")
                || msg_lower.contains("server error")
                || msg_lower.contains("connection")
                || msg_lower.contains("network")
        }
        // 다른 에러 타입은 재시도하지 않음
        _ => false,
    }
}

/// 지수 백오프 설정 생성
fn create_backoff() -> ExponentialBackoff {
    ExponentialBackoff {
        initial_interval: Duration::from_millis(500),
        max_interval: Duration::from_secs(10),
        max_elapsed_time: Some(Duration::from_secs(30)),
        multiplier: 2.0,
        ..Default::default()
    }
}

/// 재시도 로직을 적용한 비동기 작업 실행
///
/// 일시적 오류 시 지수 백오프로 재시도하고,
/// 영구적 오류 시 즉시 실패를 반환합니다.
pub async fn with_retry<F, Fut, T>(operation: F) -> Result<T, AppError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    let backoff = create_backoff();

    retry(backoff, || async {
        match operation().await {
            Ok(result) => Ok(result),
            Err(e) => {
                if is_retryable_error(&e) {
                    tracing::warn!(error = %e, "Retryable error, will retry...");
                    Err(backoff::Error::transient(e))
                } else {
                    tracing::error!(error = %e, "Permanent error, not retrying");
                    Err(backoff::Error::permanent(e))
                }
            }
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn is_retryable_error_should_return_true_for_rate_limit() {
        let error = AppError::OpenAiError("rate limit exceeded".to_string());
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_true_for_timeout() {
        let error = AppError::OpenAiError("request timeout".to_string());
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_true_for_timed_out() {
        let error = AppError::OpenAiError("connection timed out".to_string());
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_true_for_429() {
        let error = AppError::OpenAiError("HTTP 429 Too Many Requests".to_string());
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_true_for_500() {
        let error = AppError::OpenAiError("HTTP 500 Internal Server Error".to_string());
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_true_for_502() {
        let error = AppError::OpenAiError("HTTP 502 Bad Gateway".to_string());
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_true_for_503() {
        let error = AppError::OpenAiError("HTTP 503 Service Unavailable".to_string());
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_true_for_504() {
        let error = AppError::OpenAiError("HTTP 504 Gateway Timeout".to_string());
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_true_for_server_error() {
        let error = AppError::OpenAiError("server error occurred".to_string());
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_true_for_connection_error() {
        let error = AppError::OpenAiError("connection refused".to_string());
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_true_for_network_error() {
        let error = AppError::OpenAiError("network error".to_string());
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_false_for_invalid_secret_key() {
        let error = AppError::InvalidSecretKey;
        assert!(!is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_false_for_invalid_tone_style() {
        let error = AppError::InvalidToneStyle;
        assert!(!is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_false_for_validation_error() {
        let error = AppError::ValidationError("field is required".to_string());
        assert!(!is_retryable_error(&error));
    }

    #[test]
    fn is_retryable_error_should_return_false_for_auth_error_in_openai() {
        let error = AppError::OpenAiError("Invalid API key".to_string());
        assert!(!is_retryable_error(&error));
    }

    #[test]
    fn create_backoff_should_have_correct_initial_interval() {
        let backoff = create_backoff();
        assert_eq!(backoff.initial_interval, Duration::from_millis(500));
    }

    #[test]
    fn create_backoff_should_have_correct_max_interval() {
        let backoff = create_backoff();
        assert_eq!(backoff.max_interval, Duration::from_secs(10));
    }

    #[test]
    fn create_backoff_should_have_correct_max_elapsed_time() {
        let backoff = create_backoff();
        assert_eq!(backoff.max_elapsed_time, Some(Duration::from_secs(30)));
    }

    #[test]
    fn create_backoff_should_have_correct_multiplier() {
        let backoff = create_backoff();
        assert!((backoff.multiplier - 2.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn with_retry_should_succeed_on_first_try() {
        let result = with_retry(|| async { Ok::<_, AppError>("success") }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn with_retry_should_retry_on_transient_error() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = with_retry(|| {
            let attempts = Arc::clone(&attempts_clone);
            async move {
                let count = attempts.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(AppError::OpenAiError("rate limit exceeded".to_string()))
                } else {
                    Ok("success")
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn with_retry_should_not_retry_on_permanent_error() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = with_retry(|| {
            let attempts = Arc::clone(&attempts_clone);
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(AppError::InvalidSecretKey)
            }
        })
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidSecretKey));
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn with_retry_should_not_retry_validation_error() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = with_retry(|| {
            let attempts = Arc::clone(&attempts_clone);
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(AppError::ValidationError("test".to_string()))
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }
}
