# [API-030] GET /api/v1/retro-rooms/{retroRoomId}/members

회고방 멤버 목록 조회 API

## 개요

특정 회고방에 참여한 모든 멤버 목록을 조회합니다.

- 회고방에 가입된 모든 멤버 정보를 반환합니다.
- 방장(OWNER)이 먼저, 그 다음 일반 멤버(MEMBER)가 표시됩니다.
- 동일한 역할 내에서는 가입일시 기준 오름차순으로 정렬됩니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2026-02-05 | 최초 작성 |

## 엔드포인트

```
GET /api/v1/retro-rooms/{retroRoomId}/members
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
| retroRoomId | long | Yes | 조회를 원하는 회고방의 고유 ID | 1 이상의 양수 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고방 멤버 목록 조회를 성공했습니다.",
  "result": [
    {
      "memberId": 1,
      "nickname": "방장닉네임",
      "role": "OWNER",
      "joinedAt": "2026-01-01T10:00:00"
    },
    {
      "memberId": 2,
      "nickname": "첫번째멤버",
      "role": "MEMBER",
      "joinedAt": "2026-01-05T14:30:00"
    },
    {
      "memberId": 3,
      "nickname": "두번째멤버",
      "role": "MEMBER",
      "joinedAt": "2026-01-10T09:15:00"
    }
  ]
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| memberId | long | 멤버 고유 식별자 |
| nickname | string | 멤버 닉네임 |
| role | string (Enum) | 멤버 역할 |
| joinedAt | string | 회고방 가입 일시 (yyyy-MM-ddTHH:mm:ss) |

#### role Enum 값

| 값 | 설명 |
|----|------|
| OWNER | 회고방 방장 (생성자) |
| MEMBER | 일반 멤버 (초대를 통해 가입) |

> **정렬 순서**: 응답 배열은 `role` 기준 **OWNER 우선**, 동일 role 내에서는 `joinedAt` 기준 **오름차순**으로 정렬됩니다.

### 빈 결과 응답

멤버가 없는 경우 빈 배열을 반환합니다. (단, 회고방에는 최소 1명의 OWNER가 항상 존재합니다.)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고방 멤버 목록 조회를 성공했습니다.",
  "result": []
}
```

## 에러 응답

### 400 Bad Request - 잘못된 Path Parameter

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "retroRoomId가 유효한 숫자 형식이 아닙니다.",
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
  "message": "해당 회고방에 접근 권한이 없습니다.",
  "result": null
}
```

### 404 Not Found - 회고방 없음

```json
{
  "isSuccess": false,
  "code": "RETRO4041",
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
|------|-------------|-------------|----------|
| COMMON400 | 400 | 잘못된 요청 | retroRoomId가 유효한 숫자 형식이 아닌 경우 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | Authorization 헤더 누락, 토큰 만료, 잘못된 토큰 형식 |
| RETRO4031 | 403 | 해당 회고방에 접근 권한 없음 (멤버가 아님) | 요청자가 해당 회고방의 멤버가 아닌 경우 |
| RETRO4041 | 404 | 존재하지 않는 회고방 | 존재하지 않거나 삭제된 회고방의 retroRoomId로 요청 |
| COMMON500 | 500 | 멤버 목록 조회 중 서버 에러 | 데이터베이스 연결 실패, 쿼리 실행 오류 |

## 사용 예시

### cURL

```bash
curl -X GET https://api.example.com/api/v1/retro-rooms/1/members \
  -H "Authorization: Bearer {accessToken}"
```
