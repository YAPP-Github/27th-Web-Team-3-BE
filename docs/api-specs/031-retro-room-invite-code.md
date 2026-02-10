# [API-031] GET /api/v1/retro-rooms/{retroRoomId}/invite-code

회고방 초대 코드 조회 API

## 개요

특정 회고방의 초대 코드를 조회합니다.

- 회고방 생성 시 자동 생성된 초대 코드를 별도로 조회할 수 있습니다.
- 초대 코드는 생성 후 **7일간 유효**하며, 만료된 경우 `null`을 반환합니다.
- 회고방 멤버만 초대 코드를 조회할 수 있습니다.

### 초대 코드 유효 기간

| 항목 | 값 | 설명 |
|------|-------|------|
| 유효 기간 | 7일 | 생성 시점부터 7일간 유효 |
| 만료 시 | null 반환 | 만료된 코드는 조회 불가 |
| 재발급 | 추후 API 제공 예정 | 만료된 코드 재발급 기능 |

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-02-07 | 최초 작성 |

## 엔드포인트

```
GET /api/v1/retro-rooms/{retroRoomId}/invite-code
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
| retroRoomId | long | Yes | 초대 코드를 조회할 회고방 고유 ID | 1 이상의 양수 |

## Response

### 성공 (200 OK) - 유효한 초대 코드

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "초대 코드 조회를 성공했습니다.",
  "result": {
    "retroRoomId": 789,
    "inviteCode": "INV-A1B2-C3D4",
    "expiresAt": "2025-02-14T10:30:00"
  }
}
```

### 성공 (200 OK) - 만료된 초대 코드

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "초대 코드가 만료되었습니다.",
  "result": {
    "retroRoomId": 789,
    "inviteCode": null,
    "expiresAt": null
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| retroRoomId | long | 회고방 고유 ID |
| inviteCode | string \| null | 초대 코드 (형식: `INV-XXXX-XXXX`). 만료된 경우 `null` |
| expiresAt | string \| null | 초대 코드 만료 시각 (ISO 8601 형식). 만료된 경우 `null` |

## 에러 응답

### 400 Bad Request - 잘못된 Path Parameter

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "retroRoomId는 1 이상의 양수여야 합니다.",
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
  "code": "RETRO4032",
  "message": "해당 회고방에 접근 권한이 없습니다.",
  "result": null
}
```

### 404 Not Found - 회고방 없음

```json
{
  "isSuccess": false,
  "code": "RETRO4043",
  "message": "존재하지 않는 회고방입니다.",
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
| COMMON400 | 400 | 잘못된 요청 | retroRoomId가 0 이하의 값 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료 또는 잘못된 Bearer 토큰 |
| RETRO4032 | 403 | 접근 권한 없음 | 해당 회고방의 멤버가 아닌 경우 |
| RETRO4043 | 404 | 존재하지 않는 회고방 | 해당 retroRoomId의 회고방이 DB에 없음 |
| COMMON500 | 500 | 서버 내부 에러 | DB 연결 실패, 쿼리 오류 등 |

## 사용 예시

### cURL

```bash
curl -X GET https://api.example.com/api/v1/retro-rooms/789/invite-code \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}"
```

## 관련 API

- [API-005] POST /api/v1/retro-rooms - 회고방 생성 (초대 코드 자동 생성)
- [API-006] POST /api/v1/retro-rooms/join - 회고방 합류 (초대 코드 사용)
