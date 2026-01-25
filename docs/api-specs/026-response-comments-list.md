# [API-026] GET /api/v1/responses/{responseId}/comments

회고 답변 댓글 조회 API

## 개요

특정 회고 답변에 작성된 모든 댓글 리스트를 조회합니다.

- 특정 유저의 답변에 달린 동료들의 피드백이나 의견을 확인할 때 사용합니다.
- 댓글이 없는 경우 `comments` 리스트는 빈 배열(`[]`)로 반환됩니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

## 엔드포인트

```
GET /api/v1/responses/{responseId}/comments
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
| responseId | long | Yes | 댓글을 조회할 회고 답변의 고유 식별자 | 1 이상의 양수 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "댓글 조회를 성공했습니다.",
  "result": {
    "comments": [
      {
        "commentId": 789,
        "memberId": 12,
        "userName": "김민수",
        "content": "이 의견에 전적으로 동의합니다! 저도 비슷한 생각을 했어요.",
        "createdAt": "2026-01-24T16:30:15"
      }
    ]
  }
}
```

### 응답 필드

| Field | Type | Description | 제약 조건 |
|-------|------|-------------|----------|
| comments | array[object] | 댓글 리스트 (최신 순서대로 정렬) | 최소 0개, 최대 무제한 |
| comments[].commentId | long | 댓글 고유 식별자 | 양의 정수 (1 이상) |
| comments[].memberId | long | 작성자 고유 ID | 양의 정수 (1 이상) |
| comments[].userName | string | 작성자 이름(닉네임) | 1자 이상 50자 이하 |
| comments[].content | string | 댓글 내용 | 1자 이상 500자 이하 |
| comments[].createdAt | string | 작성 일시 (yyyy-MM-ddTHH:mm:ss) | ISO 8601 형식 |

### 빈 결과 응답

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "댓글 조회를 성공했습니다.",
  "result": {
    "comments": []
  }
}
```

## 에러 응답

### 400 Bad Request - 잘못된 요청

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "잘못된 요청입니다.",
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

### 403 Forbidden - 접근 권한 없음

```json
{
  "isSuccess": false,
  "code": "TEAM4031",
  "message": "해당 리소스에 접근 권한이 없습니다.",
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
|------|-------------|-------------|----------|
| COMMON400 | 400 | 잘못된 요청 | responseId가 유효한 정수가 아님 |
| AUTH4001 | 401 | 인증 실패 | 토큰 누락, 만료 또는 잘못된 형식 |
| TEAM4031 | 403 | 접근 권한 없음 | 팀 멤버가 아닌 유저가 댓글 조회 시도 |
| RES4041 | 404 | 리소스 없음 | 유효하지 않은 responseId 또는 존재하지 않는 회고 답변 |
| COMMON500 | 500 | 서버 오류 | DB 조회 중 예외 발생 또는 시스템 오류 |

## 사용 예시

### cURL

```bash
curl -X GET https://api.example.com/api/v1/responses/456/comments \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}"
```
