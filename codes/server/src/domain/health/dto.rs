use serde::Serialize;
use utoipa::ToSchema;

/// 전체 헬스 상태 응답
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    /// 서버 상태 (healthy/degraded/unhealthy)
    pub status: HealthState,
    /// 서버 버전
    #[schema(example = "0.1.0")]
    pub version: &'static str,
    /// 서버 가동 시간 (초)
    #[schema(example = 3600)]
    pub uptime_secs: u64,
    /// 의존성 체크 결과
    pub checks: HealthChecks,
}

/// 서버 상태
#[derive(Serialize, Debug, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    /// 정상 상태
    Healthy,
    /// 부분 장애 상태
    Degraded,
    /// 장애 상태
    Unhealthy,
}

/// 의존성 체크 결과 모음
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthChecks {
    /// OpenAI API 상태
    pub openai_api: CheckResult,
}

/// 개별 체크 결과
#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckResult {
    /// 체크 성공 여부
    #[schema(example = true)]
    pub status: bool,
    /// 응답 지연 시간 (ms)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 150)]
    pub latency_ms: Option<u64>,
    /// 에러 메시지 (실패 시)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl CheckResult {
    pub fn success(latency_ms: u64) -> Self {
        Self {
            status: true,
            latency_ms: Some(latency_ms),
            error: None,
        }
    }

    pub fn failure(latency_ms: u64, error: String) -> Self {
        Self {
            status: false,
            latency_ms: Some(latency_ms),
            error: Some(error),
        }
    }

    pub fn timeout() -> Self {
        Self {
            status: false,
            latency_ms: Some(5000),
            error: Some("Timeout".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_result_success_should_have_correct_fields() {
        let result = CheckResult::success(150);

        assert!(result.status);
        assert_eq!(result.latency_ms, Some(150));
        assert!(result.error.is_none());
    }

    #[test]
    fn check_result_failure_should_have_correct_fields() {
        let result = CheckResult::failure(200, "connection error".to_string());

        assert!(!result.status);
        assert_eq!(result.latency_ms, Some(200));
        assert_eq!(result.error, Some("connection error".to_string()));
    }

    #[test]
    fn check_result_timeout_should_have_correct_fields() {
        let result = CheckResult::timeout();

        assert!(!result.status);
        assert_eq!(result.latency_ms, Some(5000));
        assert_eq!(result.error, Some("Timeout".to_string()));
    }

    #[test]
    fn health_state_should_serialize_lowercase() {
        let healthy = serde_json::to_string(&HealthState::Healthy).unwrap();
        let degraded = serde_json::to_string(&HealthState::Degraded).unwrap();
        let unhealthy = serde_json::to_string(&HealthState::Unhealthy).unwrap();

        assert_eq!(healthy, "\"healthy\"");
        assert_eq!(degraded, "\"degraded\"");
        assert_eq!(unhealthy, "\"unhealthy\"");
    }

    #[test]
    fn health_status_should_serialize_with_camel_case() {
        let status = HealthStatus {
            status: HealthState::Healthy,
            version: "0.1.0",
            uptime_secs: 3600,
            checks: HealthChecks {
                openai_api: CheckResult::success(150),
            },
        };

        let json = serde_json::to_string(&status).unwrap();

        assert!(json.contains("\"uptimeSecs\""));
        assert!(json.contains("\"openaiApi\""));
        assert!(json.contains("\"latencyMs\""));
    }

    #[test]
    fn check_result_should_skip_none_fields_in_serialization() {
        let result = CheckResult::success(100);
        let json = serde_json::to_string(&result).unwrap();

        // error가 None이면 JSON에 포함되지 않아야 함
        assert!(!json.contains("error"));
    }
}
