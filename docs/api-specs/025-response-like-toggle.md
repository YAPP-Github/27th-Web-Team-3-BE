# [API-025] POST /api/responses/{responseId}/likes

회고 답변 좋아요 토글 API

## 개요

특정 회고 답변에 좋아요를 등록하거나 취소합니다.

- **상태 전이**: 좋아요 미등록 상태 → 호출 시 **등록** / 좋아요 등록 상태 → 호출 시 **취소**
- 본인의 답변에도 좋아요를 누를 수 있으며, 동일 팀 멤버 간의 긍정적인 피드백을 유도합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

## 엔드포인트

```
POST /api/responses/{responseId}/likes
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

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| responseId | long | Yes | 좋아요를 처리할 대상 답변의 고유 ID |

#### Path Parameter Validation (responseId)

| 조건 | 검증 방식 | 에러 코드 |
|------|---------|---------|
| 숫자 형식 | 경로 파라미터 타입 강제 | 400 (자동 처리) |
| 양수 | 1 이상의 값 | RES4041 |
| 존재 여부 | DB 조회 | RES4041 |
| 팀 접근 권한 | 해당 응답의 회고가 속한 팀 확인 | TEAM4031 |

### Body

별도의 Body 데이터는 필요하지 않습니다.

```json
{}
```

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "좋아요 상태가 성공적으로 업데이트되었습니다.",
  "result": {
    "responseId": 456,
    "isLiked": true,
    "totalLikes": 13
  }
}
```

### 응답 필드

| Field | Type | Description | 용도 |
|-------|------|-------------|------|
| responseId | long | 대상 답변의 ID | 요청 대상 확인 및 클라이언트 상태 동기화 |
| isLiked | boolean | 처리 후 현재 상태 (true: 좋아요 등록됨, false: 취소됨) | UI 좋아요 버튼 상태 업데이트 |
| totalLikes | integer | 업데이트된 해당 답변의 총 좋아요 개수 | 좋아요 카운트 표시 업데이트 |

## 에러 응답

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
|------|-------------|-------------|---------|
| AUTH4001 | 401 | 토큰 누락, 만료 또는 잘못된 형식 | Authorization 헤더 없음, 토큰 만료, 유효하지 않은 토큰 형식 |
| TEAM4031 | 403 | 팀 멤버가 아닌 유저가 좋아요 시도 | 요청 사용자가 해당 회고가 속한 팀의 멤버가 아닌 경우 |
| RES4041 | 404 | 유효하지 않은 responseId | responseId가 존재하지 않거나 유효하지 않은 형식 |
| COMMON500 | 500 | 동시성 제어 또는 DB 반영 중 에러 | 동시 요청 처리 중 충돌, 데이터베이스 연결 실패, 트랜잭션 롤백 |

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/responses/456/likes \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}"
```
