# [API-020] GET /api/v1/retrospects/{retrospectId}/responses

회고 답변 카테고리별 조회 API

## 개요

특정 회고 세션의 답변 리스트를 질문 카테고리별로 조회합니다.

- **커서 기반 페이지네이션(Cursor-based Pagination)**을 사용하여 무한 스크롤을 지원합니다.
- 특정 질문별로 필터링하거나 전체 답변을 한꺼번에 조회할 수 있습니다.
- 한 회고당 질문은 최대 5개까지 존재하며, 각 질문 번호에 해당하는 답변 리스트를 반환합니다.
- 데이터가 없는 경우 빈 리스트(`[]`)를 반환합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | 500 에러 추가, Query Parameter Validation, 정렬 순서, category enum 상세화, 페이징 에러 추가 |

## 엔드포인트

```
GET /api/v1/retrospects/{retrospectId}/responses
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
| retrospectId | long | Yes | 조회를 진행할 회고 세션 고유 ID | 1 이상의 양수 |

### Query Parameters

| Parameter | Type | Required | Description | Validation |
|-----------|------|----------|-------------|------------|
| category | string (Enum) | Yes | 조회 필터 | ALL, QUESTION_1~QUESTION_5 중 하나 |
| cursor | long | No | 마지막으로 조회된 답변 ID | 1 이상의 양수 (첫 요청 시 생략) |
| size | integer | No | 페이지당 조회 개수 | 1~100, 기본값: 10 |

### category Enum 설명

| Value | 한글명 | Description |
|-------|--------|-------------|
| ALL | 전체 | 모든 질문의 답변을 통합 조회 |
| QUESTION_1 | 질문 1 | 첫 번째 질문에 대한 답변만 조회 |
| QUESTION_2 | 질문 2 | 두 번째 질문에 대한 답변만 조회 |
| QUESTION_3 | 질문 3 | 세 번째 질문에 대한 답변만 조회 |
| QUESTION_4 | 질문 4 | 네 번째 질문에 대한 답변만 조회 |
| QUESTION_5 | 질문 5 | 다섯 번째 질문에 대한 답변만 조회 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "답변 리스트 조회를 성공했습니다.",
  "result": {
    "responses": [
      {
        "responseId": 501,
        "userName": "제이슨",
        "content": "이번 스프린트에서 테스트 코드를 꼼꼼히 짠 것이 좋았습니다.",
        "likeCount": 12,
        "commentCount": 3
      },
      {
        "responseId": 456,
        "userName": "김민수",
        "content": "기한 맞춰서 작업하는 것을 잘했고요...",
        "likeCount": 12,
        "commentCount": 21
      }
    ],
    "hasNext": true,
    "nextCursor": 455
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| responses | array[object] | 답변 데이터 리스트 (없을 경우 빈 배열) |
| responses[].responseId | long | 답변 고유 식별자 |
| responses[].userName | string | 작성자 이름(닉네임) |
| responses[].content | string | 답변 내용 |
| responses[].likeCount | integer | 해당 답변의 좋아요 수 |
| responses[].commentCount | integer | 해당 답변의 댓글 수 |
| hasNext | boolean | 다음 페이지 존재 여부 |
| nextCursor | long \| null | 다음 조회를 위한 커서 ID (마지막 페이지면 null) |

### 정렬 순서

| 기준 | 순서 | 설명 |
|------|------|------|
| responseId | 내림차순 | 최신 답변이 상위에 표시 |

### 빈 결과 응답

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "답변 리스트 조회를 성공했습니다.",
  "result": {
    "responses": [],
    "hasNext": false,
    "nextCursor": null
  }
}
```

## 에러 응답

### 400 Bad Request - 잘못된 Path Parameter

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "retrospectId는 1 이상의 양수여야 합니다.",
  "result": null
}
```

### 400 Bad Request - 잘못된 카테고리

```json
{
  "isSuccess": false,
  "code": "RETRO4004",
  "message": "유효하지 않은 카테고리 값입니다.",
  "result": null
}
```

### 400 Bad Request - 잘못된 페이징 파라미터

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "size는 1~100 범위의 정수여야 합니다.",
  "result": null
}
```

### 400 Bad Request - 잘못된 커서 값

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "cursor는 1 이상의 양수여야 합니다.",
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
  "message": "해당 회고에 접근 권한이 없습니다.",
  "result": null
}
```

### 404 Not Found - 회고 없음

```json
{
  "isSuccess": false,
  "code": "RETRO4041",
  "message": "존재하지 않는 회고 세션입니다.",
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
| COMMON400 | 400 | 잘못된 요청 | retrospectId/cursor가 0 이하, size가 1~100 범위 벗어남 |
| RETRO4004 | 400 | 유효하지 않은 카테고리 | category가 정의된 Enum 외의 값 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료 또는 잘못된 Bearer 토큰 |
| TEAM4031 | 403 | 접근 권한 없음 | JWT의 유저가 해당 회고가 속한 팀의 멤버가 아닌 경우 |
| RETRO4041 | 404 | 존재하지 않는 회고 세션 | 해당 retrospectId의 회고가 DB에 없음 |
| COMMON500 | 500 | 서버 내부 에러 | DB 연결 실패, 쿼리 오류 등 |

## 사용 예시

### cURL

```bash
# 전체 답변 조회 (첫 페이지)
curl -X GET "https://api.example.com/api/v1/retrospects/100/responses?category=ALL&size=10" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}"

# 질문 1에 대한 답변 조회 (커서 기반 다음 페이지)
curl -X GET "https://api.example.com/api/v1/retrospects/100/responses?category=QUESTION_1&cursor=455&size=10" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}"
```
