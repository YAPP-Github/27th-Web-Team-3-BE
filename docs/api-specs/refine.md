# POST /api/ai/retrospective/refine

회고 말투 정제 API

## 개요

작성된 회고 내용을 선택한 말투 스타일(상냥체/정중체)로 정제합니다.

## 엔드포인트

```
POST /api/ai/retrospective/refine
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
  "content": "오늘 일 힘들었음",
  "toneStyle": "KIND",
  "secretKey": "your-secret-key"
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| content | string | Yes | 정제할 회고 내용 | 1 ~ 5000자 |
| toneStyle | string | Yes | 말투 스타일 | KIND 또는 POLITE |
| secretKey | string | Yes | API 인증 키 | 1자 이상 |

### ToneStyle 값

| Value | Description | Example |
|-------|-------------|---------|
| KIND | 상냥체 | "오늘 힘들었어요~" |
| POLITE | 정중체 | "오늘 힘들었습니다." |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "originalContent": "오늘 일 힘들었음",
    "refinedContent": "오늘 일이 많이 힘들었어요.",
    "toneStyle": "KIND"
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| originalContent | string | 원본 회고 내용 |
| refinedContent | string | 정제된 회고 내용 |
| toneStyle | string | 적용된 말투 스타일 |

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

### 400 Bad Request - 잘못된 말투 스타일

```json
{
  "isSuccess": false,
  "code": "AI_002",
  "message": "유효하지 않은 말투 스타일입니다. KIND 또는 POLITE만 가능합니다.",
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
| AI_002 | 400 | 유효하지 않은 말투 스타일 |
| AI_003 | 500 | AI 서비스 연결 실패 (API 키 문제) |
| AI_004 | 503 | AI 서비스 요청 한도 초과 |
| AI_005 | 503 | AI 서비스 일시적 오류 |
| AI_006 | 500 | AI 서비스 일반 오류 |
| COMMON400 | 400 | 잘못된 요청 (필드 누락, 유효성 검증 실패) |
| COMMON500 | 500 | 서버 내부 에러 |
| RATE_001 | 429 | 요청 한도 초과 |

## 사용 예시

### cURL - 상냥체로 정제

```bash
curl -X POST https://api.example.com/api/ai/retrospective/refine \
  -H "Content-Type: application/json" \
  -d '{
    "content": "오늘 일 힘들었음",
    "toneStyle": "KIND",
    "secretKey": "your-secret-key"
  }'
```

### cURL - 정중체로 정제

```bash
curl -X POST https://api.example.com/api/ai/retrospective/refine \
  -H "Content-Type: application/json" \
  -d '{
    "content": "오늘 일 힘들었음",
    "toneStyle": "POLITE",
    "secretKey": "your-secret-key"
  }'
```

## 관련 파일

- Handler: `codes/server/src/domain/ai/handler.rs`
- DTO: `codes/server/src/domain/ai/dto.rs`
- Service: `codes/server/src/domain/ai/service.rs`
