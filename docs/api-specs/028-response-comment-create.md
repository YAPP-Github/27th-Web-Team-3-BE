# [API-028] POST /api/v1/responses/{responseId}/comments

회고 답변 댓글 작성 API

## 개요

동료의 회고 답변에 댓글(의견)을 남깁니다.

- 댓글 내용은 **최대 200자**까지 작성이 가능하며, 빈 값은 허용되지 않습니다.
- 작성된 댓글의 고유 ID와 생성 시간을 즉시 반환하여 클라이언트 UI 업데이트를 돕습니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

## 엔드포인트

```
POST /api/v1/responses/{responseId}/comments
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
| responseId | long | Yes | 댓글을 작성할 대상 답변의 고유 ID | 1 이상의 양수 |

### Body

```json
{
  "content": "이 부분 정말 공감되네요! 고생 많으셨습니다."
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| content | string | Yes | 댓글 내용 | 최대 200자 |

#### content 검증 규칙

| 규칙 | 조건 | 에러 코드 |
|------|------|---------|
| 필수 입력 | 빈 문자열 또는 null은 허용 안함 | COMMON400 |
| 최대 길이 | 200자 초과 | RES4001 |
| 문자열 타입 | 유효한 UTF-8 문자열 | COMMON400 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "댓글이 성공적으로 등록되었습니다.",
  "result": {
    "commentId": 789,
    "responseId": 456,
    "content": "이 부분 정말 공감되네요! 고생 많으셨습니다.",
    "createdAt": "2026-01-24T15:48:21"
  }
}
```

### 응답 필드

| Field | Type | 용도 | Description |
|-------|------|------|-------------|
| commentId | long | 클라이언트 ID 관리 | 생성된 댓글의 고유 ID (UI 업데이트 및 향후 수정/삭제에 사용) |
| responseId | long | 데이터 검증 | 부모 답변의 ID (요청 시 전달한 값과 일치 확인) |
| content | string | 내용 확인 | 서버가 저장한 댓글 내용 (요청 시 전달한 값과 일치 확인) |
| createdAt | string | UI 표시 | 작성 일시 (yyyy-MM-ddTHH:mm:ss 형식, 클라이언트 UI에 표시) |

## 에러 응답

### 400 Bad Request - 필드 누락 또는 빈 값

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "요청 본문의 필수 필드가 누락되었거나 유효하지 않습니다.",
  "result": null
}
```

### 400 Bad Request - 댓글 길이 초과

```json
{
  "isSuccess": false,
  "code": "RES4001",
  "message": "댓글은 최대 200자까지만 입력 가능합니다.",
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

### 403 Forbidden - 권한 없음

```json
{
  "isSuccess": false,
  "code": "TEAM4031",
  "message": "댓글 작성 권한이 없습니다.",
  "result": null
}
```

### 404 Not Found - 답변 없음

```json
{
  "isSuccess": false,
  "code": "RES4041",
  "message": "존재하지 않는 회고 답변입니다.",
  "result": null
}
```

### 500 Internal Server Error - 서버 오류

```json
{
  "isSuccess": false,
  "code": "COMMON500",
  "message": "서버 내부 오류가 발생했습니다.",
  "result": null
}
```

## 에러 코드 요약

| Code | HTTP Status | Description | 발생 조건 |
|------|-------------|-------------|---------|
| COMMON400 | 400 | 잘못된 요청 | content 필드 누락, null 또는 빈 문자열 |
| RES4001 | 400 | 댓글 길이 초과 | 댓글 내용이 200자를 초과 |
| AUTH4001 | 401 | 인증 실패 | Authorization 헤더 누락, 만료 또는 잘못된 토큰 형식 |
| TEAM4031 | 403 | 권한 없음 | 요청 유저가 해당 팀의 멤버가 아닌 경우 |
| RES4041 | 404 | 답변 없음 | 유효하지 않은 responseId 또는 존재하지 않는 답변 |
| COMMON500 | 500 | 서버 내부 오류 | 데이터베이스 오류 등 예상 밖의 서버 오류 발생 |

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/v1/responses/456/comments \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "content": "이 부분 정말 공감되네요! 고생 많으셨습니다."
  }'
```
