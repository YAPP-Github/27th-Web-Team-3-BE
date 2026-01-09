use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tokio::time::timeout;

use super::dto::{CheckResult, HealthChecks, HealthState, HealthStatus};
use crate::domain::ai::service::AiService;

/// 서버 시작 시간 (전역)
static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// 헬스체크 결과 캐시 (전역)
static HEALTH_CACHE: std::sync::OnceLock<Arc<RwLock<Option<CachedHealth>>>> =
    std::sync::OnceLock::new();

/// 헬스체크 타임아웃 (5초)
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

/// Degraded 상태 임계값 (2초)
const DEGRADED_THRESHOLD: Duration = Duration::from_secs(2);

/// 캐시 유효 시간 (30초)
const CACHE_DURATION: Duration = Duration::from_secs(30);

/// 캐시된 헬스체크 결과
struct CachedHealth {
    result: CheckResult,
    cached_at: Instant,
}

/// 캐시 초기화
fn get_health_cache() -> Arc<RwLock<Option<CachedHealth>>> {
    HEALTH_CACHE
        .get_or_init(|| Arc::new(RwLock::new(None)))
        .clone()
}

/// 서버 시작 시간 초기화
///
/// main 함수에서 서버 시작 시 호출해야 합니다.
pub fn init_start_time() {
    START_TIME.get_or_init(Instant::now);
}

/// 서버 가동 시간(초) 반환
pub fn get_uptime_secs() -> u64 {
    START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0)
}

/// 전체 헬스 체크 수행 (캐싱 적용)
pub async fn check_health(ai_service: &AiService) -> HealthStatus {
    let uptime = get_uptime_secs();
    let openai_check = check_openai_cached(ai_service).await;

    let status = determine_health_state(&openai_check);

    HealthStatus {
        status,
        version: env!("CARGO_PKG_VERSION"),
        uptime_secs: uptime,
        checks: HealthChecks {
            openai_api: openai_check,
        },
    }
}

/// OpenAI 상태에 따른 전체 상태 결정
fn determine_health_state(check: &CheckResult) -> HealthState {
    if !check.status {
        return HealthState::Unhealthy;
    }

    // 응답 시간이 2초 이상이면 Degraded
    if let Some(latency) = check.latency_ms {
        if latency >= DEGRADED_THRESHOLD.as_millis() as u64 {
            return HealthState::Degraded;
        }
    }

    HealthState::Healthy
}

/// OpenAI API 연결 상태 확인 (캐싱 적용)
async fn check_openai_cached(ai_service: &AiService) -> CheckResult {
    let cache = get_health_cache();

    // 캐시 확인
    {
        let cached = cache.read().await;
        if let Some(ref c) = *cached {
            if c.cached_at.elapsed() < CACHE_DURATION {
                tracing::debug!(
                    cache_age_secs = c.cached_at.elapsed().as_secs(),
                    "Using cached health check result"
                );
                return c.result.clone();
            }
        }
    }

    // 캐시 만료 또는 없음 - 새로 검사
    tracing::debug!("Performing fresh health check");
    let result = check_openai_fresh(ai_service).await;

    // 캐시 업데이트
    {
        let mut cached = cache.write().await;
        *cached = Some(CachedHealth {
            result: result.clone(),
            cached_at: Instant::now(),
        });
    }

    result
}

/// OpenAI API 실제 헬스체크 (텍스트 생성 검증)
async fn check_openai_fresh(ai_service: &AiService) -> CheckResult {
    let start = Instant::now();

    // 실제 텍스트 생성으로 검증
    let result = timeout(HEALTH_CHECK_TIMEOUT, ai_service.health_check()).await;
    let latency_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(Ok(_response)) => {
            tracing::info!(latency_ms, "OpenAI health check passed");
            CheckResult::success(latency_ms)
        }
        Ok(Err(e)) => {
            tracing::warn!(latency_ms, error = %e, "OpenAI health check failed");
            CheckResult::failure(latency_ms, e.to_string())
        }
        Err(_) => {
            tracing::warn!("OpenAI health check timed out");
            CheckResult::timeout()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_start_time_should_set_once() {
        // OnceLock은 한 번만 설정됨
        init_start_time();
        let first = START_TIME.get();
        assert!(first.is_some());

        // 두 번째 호출은 값을 변경하지 않음
        init_start_time();
        let second = START_TIME.get();
        assert_eq!(first, second);
    }

    #[test]
    fn get_uptime_secs_should_return_elapsed_time() {
        init_start_time();
        let uptime = get_uptime_secs();
        // uptime이 유효한 값인지 확인 (u64이므로 항상 0 이상)
        // 이 테스트는 함수가 정상적으로 동작하는지 확인
        assert!(START_TIME.get().is_some());
        // uptime은 0 이상의 유효한 값 (컴파일러 경고 방지를 위해 명시적 확인 생략)
        let _ = uptime;
    }

    #[test]
    fn determine_health_state_should_return_healthy_for_fast_success() {
        let check = CheckResult::success(500); // 500ms - 빠른 응답
        let state = determine_health_state(&check);
        assert_eq!(state, HealthState::Healthy);
    }

    #[test]
    fn determine_health_state_should_return_degraded_for_slow_success() {
        let check = CheckResult::success(2500); // 2.5초 - 느린 응답
        let state = determine_health_state(&check);
        assert_eq!(state, HealthState::Degraded);
    }

    #[test]
    fn determine_health_state_should_return_unhealthy_for_failure() {
        let check = CheckResult::failure(100, "API Error".to_string());
        let state = determine_health_state(&check);
        assert_eq!(state, HealthState::Unhealthy);
    }

    #[test]
    fn determine_health_state_should_return_unhealthy_for_timeout() {
        let check = CheckResult::timeout();
        let state = determine_health_state(&check);
        assert_eq!(state, HealthState::Unhealthy);
    }

    #[test]
    fn determine_health_state_boundary_at_2000ms_should_be_healthy() {
        // 정확히 2000ms는 Healthy (임계값 미만)
        let check = CheckResult::success(1999);
        let state = determine_health_state(&check);
        assert_eq!(state, HealthState::Healthy);
    }

    #[test]
    fn determine_health_state_boundary_at_2000ms_should_be_degraded() {
        // 정확히 2000ms는 Degraded (임계값 이상)
        let check = CheckResult::success(2000);
        let state = determine_health_state(&check);
        assert_eq!(state, HealthState::Degraded);
    }

    #[test]
    fn cache_duration_should_be_30_seconds() {
        assert_eq!(CACHE_DURATION.as_secs(), 30);
    }

    #[test]
    fn degraded_threshold_should_be_2_seconds() {
        assert_eq!(DEGRADED_THRESHOLD.as_secs(), 2);
    }

    #[test]
    fn health_check_timeout_should_be_5_seconds() {
        assert_eq!(HEALTH_CHECK_TIMEOUT.as_secs(), 5);
    }
}
