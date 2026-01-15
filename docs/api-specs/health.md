# GET /health

헬스체크 API

## 개요

서버 상태, 버전, 가동 시간, 의존성 상태를 반환합니다. 로드밸런서나 모니터링 시스템에서 서버 상태를 확인하는 데 사용됩니다.

## 엔드포인트

```
GET /health
```

## 인증

- 인증 불필요

## Request

### Headers

없음

### Parameters

없음

## Response

### 성공 (200 OK) - 정상 상태

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptimeSecs": 3600,
  "checks": {
    "openaiApi": {
      "status": true,
      "latencyMs": 150
    }
  }
}
```

### 성공 (200 OK) - 부분 장애 상태

```json
{
  "status": "degraded",
  "version": "0.1.0",
  "uptimeSecs": 3600,
  "checks": {
    "openaiApi": {
      "status": false,
      "latencyMs": 5000,
      "error": "Timeout"
    }
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| status | string | 서버 상태 (healthy / degraded / unhealthy) |
| version | string | 서버 버전 |
| uptimeSecs | number | 서버 가동 시간 (초) |
| checks | object | 의존성 체크 결과 |
| checks.openaiApi | object | OpenAI API 상태 |
| checks.openaiApi.status | boolean | 체크 성공 여부 |
| checks.openaiApi.latencyMs | number | 응답 지연 시간 (ms) |
| checks.openaiApi.error | string? | 에러 메시지 (실패 시에만 포함) |

### 서버 상태 값

| Value | Description |
|-------|-------------|
| healthy | 모든 의존성 정상 |
| degraded | 일부 의존성 장애 (서비스 제한적 동작) |
| unhealthy | 주요 의존성 장애 (서비스 불가) |

## 사용 예시

### cURL

```bash
curl -X GET https://api.example.com/health
```

### 로드밸런서 설정 예시 (AWS ALB)

```yaml
HealthCheck:
  Path: /health
  Protocol: HTTP
  HealthyThresholdCount: 2
  UnhealthyThresholdCount: 3
  IntervalSeconds: 30
  TimeoutSeconds: 5
```

### Kubernetes Probe 설정 예시

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 30

readinessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
```

## 관련 파일

- Handler: `codes/server/src/domain/health/handler.rs`
- DTO: `codes/server/src/domain/health/dto.rs`
- Service: `codes/server/src/domain/health/service.rs`
