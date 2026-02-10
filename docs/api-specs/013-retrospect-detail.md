# [API-013] GET /api/v1/retrospects/{retrospectId}

회고 상세 정보 조회 API

## 개요

특정 회고 세션의 상세 정보(제목, 일시, 유형, 참여 멤버, 질문 리스트 및 전체 통계)를 조회합니다.

- 페이지 상단 레이아웃 구성을 위한 핵심 데이터를 제공합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | Path Parameter 검증, Enum 설명, 정렬 순서 추가 |
| 1.2.0 | 2025-01-25 | retroRoomId 필드 추가, 날짜 포맷 ISO 8601(YYYY-MM-DD) 통일 |
| 1.3.0 | 2025-02-07 | currentUserStatus 필드 추가 (현재 유저의 제출 상태) |

## 엔드포인트

```
GET /api/v1/retrospects/{retrospectId}
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

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 상세 정보 조회를 성공했습니다.",
  "result": {
    "retroRoomId": 789,
    "title": "3차 스프린트 회고",
    "startTime": "2026-01-24",
    "retroCategory": "KPT",
    "currentUserStatus": "SUBMITTED",
    "members": [
      { "memberId": 1, "userName": "김민철" },
      { "memberId": 2, "userName": "카이" }
    ],
    "totalLikeCount": 156,
    "totalCommentCount": 42,
    "questions": [
      {
        "index": 1,
        "content": "계속 유지하고 싶은 좋은 점은 무엇인가요?"
      },
      {
        "index": 2,
        "content": "개선이 필요한 문제점은 무엇인가요?"
      },
      {
        "index": 3,
        "content": "다음에 시도해보고 싶은 것은 무엇인가요?"
      }
    ]
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| retroRoomId | long | 회고가 속한 회고방의 고유 ID |
| title | string | 회고 제목 (프로젝트명) |
| startTime | string | 회고 시작 날짜 (YYYY-MM-DD) |
| retroCategory | string (Enum) | 회고 유형 |
| currentUserStatus | string (Enum) \| null | 현재 로그인 유저의 제출 상태 (하단 Enum 참조). 회고 미참여 시 `null` |
| members | array[object] | 참여 멤버 리스트 (참석 등록일 기준 오름차순 정렬) |
| members[].memberId | long | 멤버 고유 식별자 |
| members[].userName | string | 멤버 이름 (닉네임) |
| totalLikeCount | integer | 회고 전체 좋아요 합계 |
| totalCommentCount | integer | 회고 전체 댓글 합계 |
| questions | array[object] | 해당 회고의 질문 리스트 (index 기준 오름차순 정렬, 최대 5개) |
| questions[].index | integer | 질문 순서 (1~5) |
| questions[].content | string | 질문 내용 (회고 생성 시 retrospectMethod에 따라 자동 생성) |

### retroCategory Enum 설명

| Value | 한글명 | Description |
|-------|--------|-------------|
| KPT | Keep-Problem-Try | 유지할 점, 문제점, 시도할 점 정리 방식 |
| FOUR_L | 4L | Liked-Learned-Lacked-Longed for 방식 |
| FIVE_F | 5F | Facts-Feelings-Findings-Future-Feedback 방식 |
| PMI | Plus-Minus-Interesting | 긍정-부정-흥미로운 점 분류 방식 |
| FREE | 자유 형식 | 형식 제약 없는 자유 작성 |

### currentUserStatus Enum 설명

| Value | 한글명 | Description |
|-------|--------|-------------|
| DRAFT | 임시저장 | 회고 답변을 임시 저장한 상태 |
| SUBMITTED | 제출완료 | 회고 답변을 최종 제출한 상태 |
| ANALYZED | 분석완료 | AI 분석이 완료된 상태 |
| null | 미참여 | 회고에 참여하지 않은 상태 (member_retro 레코드 없음) |

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
  "message": "해당 회고에 접근 권한이 없습니다.",
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
| RETRO4031 | 403 | 접근 권한 없음 | 해당 회고가 속한 회고방의 멤버가 아닌 경우 |
| RETRO4041 | 404 | 존재하지 않는 회고 | 해당 retrospectId의 회고가 DB에 없음 |
| COMMON500 | 500 | 서버 내부 에러 | DB 연결 실패, 쿼리 오류 등 |

## 사용 예시

### cURL

```bash
curl -X GET https://api.example.com/api/v1/retrospects/100 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}"
```
