# POST /api/ai/retrospective/guide

회고 작성 가이드 API

## 개요

현재 작성 중인 회고 내용을 분석하여 AI가 작성 가이드 메시지를 제공합니다.

## 엔드포인트

```
POST /api/ai/retrospective/guide
```

## 인증

- `secretKey` 필드를 통한 API 키 인증

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Content-Type | application/json | Yes |

### Body

```json
{
  "currentContent": "오늘 프로젝트를 진행하면서 새로운 기술을 배웠다.",
  "secretKey": "your-secret-key"
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| currentContent | string | Yes | 현재 작성 중인 회고 내용 | 1 ~ 5000자 |
| secretKey | string | Yes | API 인증 키 | 1자 이상 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "currentContent": "오늘 프로젝트를 진행하면서 새로운 기술을 배웠다.",
    "guideMessage": "좋은 시작이에요! 어떤 기술을 배우셨나요? 그 기술을 배우게 된 계기와 느낀 점도 함께 적어보세요."
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| currentContent | string | 원본 회고 내용 |
| guideMessage | string | AI가 생성한 가이드 메시지 |

## 에러 응답

### 400 Bad Request - 유효성 검증 실패

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "잘못된 요청입니다: 내용은 1자 이상 5000자 이하여야 합니다",
  "result": null
}
```

### 401 Unauthorized - 인증 실패

```json
{
  "isSuccess": false,
  "code": "AI_001",
  "message": "유효하지 않은 비밀 키입니다.",
  "result": null
}
```

### 429 Too Many Requests - 요청 한도 초과

```json
{
  "isSuccess": false,
  "code": "RATE_001",
  "message": "요청이 너무 많습니다. 잠시 후 다시 시도해주세요.",
  "result": null
}
```

### 500 Internal Server Error - 서버 에러

```json
{
  "isSuccess": false,
  "code": "COMMON500",
  "message": "서버 에러, 관리자에게 문의 바랍니다.",
  "result": null
}
```

### 503 Service Unavailable - AI 서비스 일시 불안정

```json
{
  "isSuccess": false,
  "code": "AI_005",
  "message": "AI 서비스가 일시적으로 불안정합니다. 잠시 후 다시 시도해주세요.",
  "result": null
}
```

## 에러 코드 요약

| Code | HTTP Status | Description |
|------|-------------|-------------|
| AI_001 | 401 | 유효하지 않은 비밀 키 |
| AI_003 | 500 | AI 서비스 연결 실패 (API 키 문제) |
| AI_004 | 503 | AI 서비스 요청 한도 초과 |
| AI_005 | 503 | AI 서비스 일시적 오류 |
| AI_006 | 500 | AI 서비스 일반 오류 |
| COMMON400 | 400 | 잘못된 요청 (필드 누락, 유효성 검증 실패) |
| COMMON500 | 500 | 서버 내부 에러 |
| RATE_001 | 429 | 요청 한도 초과 |

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/ai/retrospective/guide \
  -H "Content-Type: application/json" \
  -d '{
    "currentContent": "오늘 프로젝트를 진행하면서 새로운 기술을 배웠다.",
    "secretKey": "your-secret-key"
  }'
```

## 관련 파일

- Handler: `codes/server/src/domain/ai/handler.rs`
- DTO: `codes/server/src/domain/ai/dto.rs`
- Service: `codes/server/src/domain/ai/service.rs`
