# [API-015] GET /api/v1/retrospects/{retrospectId}/participants

회고 참여자 및 질문 조회 API

## 개요

특정 회고에 참여 등록된 인원들의 목록, 총 인원수, 그리고 해당 회고 방식(KPT, 4L 등)에 따라 할당된 전체 질문 리스트를 한꺼번에 조회합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | 500 에러 추가, Path Parameter 검증, 배열 정렬 순서, 에러 조건 상세화 |
| 1.2.0 | 2025-01-25 | 질문 생성 로직 명시 (회고 방식별 기본 질문) |

## 엔드포인트

```
GET /api/v1/retrospects/{retrospectId}/participants
```

## 인증

- `Authorization` 헤더를 통한 Bearer 토큰 인증

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Authorization | Bearer {accessToken} | Yes |

### Path Parameters

| Parameter | Type | Required | Description | Validation |
|-----------|------|----------|-------------|------------|
| retrospectId | long | Yes | 조회를 원하는 회고의 고유 ID | 1 이상의 양수 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 참여자 및 질문 정보를 성공적으로 조회했습니다.",
  "result": {
    "totalCount": 3,
    "participants": [
      { "nickname": "제이슨" },
      { "nickname": "알렉스" },
      { "nickname": "사라" }
    ],
    "questions": [
      {
        "questionId": 1,
        "content": "계속 유지하고 싶은 좋은 점은 무엇인가요?"
      },
      {
        "questionId": 2,
        "content": "개선이 필요한 문제점은 무엇인가요?"
      },
      {
        "questionId": 3,
        "content": "다음에 시도해보고 싶은 것은 무엇인가요?"
      },
      {
        "questionId": 4,
        "content": "전체 프로젝트를 돌아보며 느낀 점을 자유롭게 작성해주세요."
      },
      {
        "questionId": 5,
        "content": "추가로 공유하고 싶은 의견이 있나요?"
      }
    ]
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| totalCount | integer | 총 참여 등록 인원수 |
| participants | array[object] | 참여자 목록 (참석 등록일 기준 오름차순 정렬) |
| participants[].nickname | string | 참여자 닉네임 |
| questions | array[object] | 해당 회고의 질문 리스트 (questionId 기준 오름차순 정렬, 정확히 5개) |
| questions[].questionId | long | 질문 고유 식별자 |
| questions[].content | string | 질문 내용 (회고 생성 시 retrospectMethod에 따라 자동 생성, 상세 내용은 API-011 참조) |

### 정렬 순서

| 필드 | 정렬 기준 | 순서 |
|------|----------|------|
| participants | 참석 등록일시 (createdAt) | 오름차순 (먼저 등록한 사람이 상위) |
| questions | 질문 ID (questionId) | 오름차순 (1번 질문부터 순서대로) |

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
  "code": "RETRO4031",
  "message": "해당 회고방의 멤버가 아닙니다.",
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
| COMMON400 | 400 | 잘못된 요청 | retrospectId가 0 이하의 값 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료 또는 잘못된 Bearer 토큰 |
| RETRO4031 | 403 | 접근 권한 없음 | JWT의 유저가 해당 회고가 속한 회고방의 멤버가 아닌 경우 |
| RETRO4041 | 404 | 존재하지 않는 회고 | 해당 retrospectId의 회고가 DB에 없음 |
| COMMON500 | 500 | 서버 내부 에러 | DB 연결 실패, 쿼리 오류 등 |

## 사용 예시

### cURL

```bash
curl -X GET https://api.example.com/api/v1/retrospects/100/participants \
  -H "Authorization: Bearer {accessToken}"
```
