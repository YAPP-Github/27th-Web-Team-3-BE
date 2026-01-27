# [API-006] GET /api/v1/retro-rooms

참여 회고방 목록 조회 API

## 개요

현재 로그인한 사용자가 참여 중인 모든 회고방 목록을 조회합니다.

- 사용자가 설정한 정렬 순서(`orderIndex`)가 반영되어 반환됩니다.
- 참여 중인 회고방이 없는 경우 `result`는 빈 배열(`[]`)로 반환됩니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

## 엔드포인트

```
GET /api/v1/retro-rooms
```

## 인증

- `Authorization` 헤더를 통한 Bearer 토큰 인증

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Authorization | Bearer {accessToken} | Yes |

### Body

없음 (GET 요청)

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "참여 중인 회고방 목록 조회를 성공했습니다.",
  "result": [
    {
      "retroRoomId": 789,
      "retroRoomName": "가장 먼저 만든 회고방",
      "orderIndex": 1
    },
    {
      "retroRoomId": 456,
      "retroRoomName": "두 번째로 만든 회고방",
      "orderIndex": 2
    }
  ]
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| retroRoomId | long | 회고방 고유 식별자 |
| retroRoomName | string | 회고방 이름 |
| orderIndex | integer | 정렬 순서 (1부터 시작, 낮을수록 상단에 노출) |

> **정렬 순서**: 응답 배열은 `orderIndex` 기준 **오름차순**으로 정렬되어 반환됩니다.

### 빈 결과 응답

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "참여 중인 회고방 목록 조회를 성공했습니다.",
  "result": []
}
```

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
| AUTH4001 | 401 | 토큰 누락, 만료 또는 유효하지 않은 JWT | Authorization 헤더 누락, 토큰 만료, 잘못된 토큰 형식 |
| COMMON500 | 500 | 데이터 조회 중 DB 에러 등 서버 내부 문제 | 데이터베이스 연결 실패, 쿼리 실행 오류 |

## 사용 예시

### cURL

```bash
curl -X GET https://api.example.com/api/v1/retro-rooms \
  -H "Authorization: Bearer {accessToken}"
```
