# [API-017] POST /api/retrospects/{retrospectId}/submit

회고 최종 제출 API

## 개요

작성한 모든 답변(총 5개)을 최종 제출합니다.

- 각 답변은 **최대 1,000자**까지 입력 가능합니다.
- 제출 완료 시 회고 상태가 `SUBMITTED`(작성 완료)로 변경되며, 이후 수정이 제한될 수 있습니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | 500 에러 추가, status enum 상세 설명, 응답 필드 설명 보완 |

## 엔드포인트

```
POST /api/retrospects/{retrospectId}/submit
```

## 인증

- `Authorization` 헤더를 통한 Bearer 토큰 인증

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Content-Type | application/json | Yes |
| Authorization | Bearer {accessToken} | Yes |

### Path Parameters

| Parameter | Type | Required | Description | Validation |
|-----------|------|----------|-------------|------------|
| retrospectId | long | Yes | 제출할 회고의 고유 식별자 | 1 이상의 양수 |

### Body

```json
{
  "answers": [
    { "questionNumber": 1, "content": "유지할 점에 대한 답변..." },
    { "questionNumber": 2, "content": "문제점에 대한 답변..." },
    { "questionNumber": 3, "content": "시도할 점에 대한 답변..." },
    { "questionNumber": 4, "content": "느낀 점에 대한 답변..." },
    { "questionNumber": 5, "content": "기타 의견에 대한 답변..." }
  ]
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| answers | array[object] | Yes | 제출할 답변 리스트 | 정확히 5개 항목 필수 |
| answers[].questionNumber | integer | Yes | 질문 번호 | 1 ~ 5 (모든 번호 필수) |
| answers[].content | string | Yes | 답변 내용 | 1~1,000자, 공백만 입력 불가 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 제출이 성공적으로 완료되었습니다.",
  "result": {
    "retrospectId": 101,
    "submittedAt": "2026-01-24T17:00:00",
    "status": "SUBMITTED"
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| retrospectId | long | 제출된 회고의 고유 ID |
| submittedAt | string | 최종 제출 일시 (yyyy-MM-ddTHH:mm:ss) |
| status | string (Enum) | 현재 회고 상태 |

### status Enum 설명

| Value | 한글명 | Description |
|-------|--------|-------------|
| DRAFT | 임시 저장 | 작성 중인 상태 (임시 저장만 완료) |
| SUBMITTED | 제출 완료 | 모든 답변 최종 제출 완료 |
| ANALYZED | 분석 완료 | AI 분석이 완료된 상태 |

**참고**: 이 API 호출 성공 시 항상 `SUBMITTED` 상태로 변경됩니다.

## 에러 응답

### 400 Bad Request - 답변 누락

```json
{
  "isSuccess": false,
  "code": "RETRO4002",
  "message": "모든 질문에 대한 답변이 필요합니다.",
  "result": null
}
```

### 400 Bad Request - 답변 길이 초과

```json
{
  "isSuccess": false,
  "code": "RETRO4003",
  "message": "답변은 1,000자를 초과할 수 없습니다.",
  "result": null
}
```

### 400 Bad Request - 공백만 입력

```json
{
  "isSuccess": false,
  "code": "RETRO4007",
  "message": "답변 내용은 공백만으로 구성될 수 없습니다.",
  "result": null
}
```

### 401 Unauthorized - 인증 실패

```json
{
  "isSuccess": false,
  "code": "AUTH4001",
  "message": "인증 정보가 유효하지 않습니다.",
  "result": null
}
```

### 403 Forbidden - 이미 제출됨

```json
{
  "isSuccess": false,
  "code": "RETRO4033",
  "message": "이미 제출이 완료된 회고입니다.",
  "result": null
}
```

### 404 Not Found - 회고 없음

```json
{
  "isSuccess": false,
  "code": "RETRO4041",
  "message": "존재하지 않는 회고입니다.",
  "result": null
}
```

### 500 Internal Server Error - 서버 에러

```json
{
  "isSuccess": false,
  "code": "COMMON500",
  "message": "서버 내부 오류입니다.",
  "result": null
}
```

## 에러 코드 요약

| Code | HTTP Status | Description | 발생 조건 |
|------|-------------|-------------|-----------|
| RETRO4002 | 400 | 답변 누락 | answers 배열이 5개가 아니거나 특정 질문 번호 누락 |
| RETRO4003 | 400 | 답변 글자 수 제한 초과 | content가 1,000자 초과 |
| RETRO4007 | 400 | 공백만 입력 | content가 공백 문자만으로 구성됨 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료 또는 잘못된 Bearer 토큰 |
| RETRO4033 | 403 | 중복 제출 방지 | 이미 SUBMITTED 또는 ANALYZED 상태인 회고 |
| RETRO4041 | 404 | 존재하지 않는 회고 | 해당 retrospectId의 회고가 DB에 없음 |
| COMMON500 | 500 | 서버 내부 에러 | DB 연결 실패, 트랜잭션 오류 등 |

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/retrospects/101/submit \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "answers": [
      { "questionNumber": 1, "content": "유지할 점에 대한 답변..." },
      { "questionNumber": 2, "content": "문제점에 대한 답변..." },
      { "questionNumber": 3, "content": "시도할 점에 대한 답변..." },
      { "questionNumber": 4, "content": "느낀 점에 대한 답변..." },
      { "questionNumber": 5, "content": "기타 의견에 대한 답변..." }
    ]
  }'
```
