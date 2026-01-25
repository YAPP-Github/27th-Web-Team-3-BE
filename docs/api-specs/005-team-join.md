# [API-005] POST /api/v1/teams/join

팀 합류 API (초대 링크)

## 개요

사용자가 클릭하거나 복사한 **초대 링크 전체**를 전달받아 해당 팀의 멤버로 합류합니다.

- 서버는 전달받은 URL에서 초대 식별값(`inviteCode`)을 추출하여 유효성을 검사합니다.
- 이미 가입된 유저이거나 만료된 링크인 경우 적절한 에러 코드를 반환합니다.

### 초대 코드 만료 정책

- **유효 기간**: 초대 코드 생성 시점으로부터 **7일간** 유효합니다.
- **만료 처리**: 만료된 초대 코드로 팀 참가 시도 시 `TEAM4003` 에러를 반환합니다.
- **재발급**: 만료된 초대 코드는 팀 관리자가 새로운 코드를 재발급할 수 있습니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

## 엔드포인트

```
POST /api/v1/teams/join
```

## 인증

- `Authorization` 헤더를 통한 Bearer 토큰 인증

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Content-Type | application/json | Yes |
| Authorization | Bearer {accessToken} | Yes |

### Body

```json
{
  "inviteUrl": "https://service.com/invite/INV-A1B2-C3D4"
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| inviteUrl | string | Yes | 사용자가 접속한 전체 초대 링크 URL | URL 형식 필수 |

> **inviteCode 추출 로직**: 서버는 `inviteUrl`의 마지막 경로 세그먼트에서 초대 코드를 추출합니다.
> - 예: `https://service.com/invite/INV-A1B2-C3D4` → `INV-A1B2-C3D4`
> - 쿼리 파라미터 형식도 지원: `https://service.com/join?code=INV-A1B2-C3D4`

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "팀에 성공적으로 합류하였습니다.",
  "result": {
    "teamId": 789,
    "teamName": "코드 마스터즈",
    "joinedAt": "2026-01-24T15:45:00"
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| teamId | long | 합류한 팀 고유 ID |
| teamName | string | 합류한 팀 이름 |
| joinedAt | string | 팀 합류 일시 (yyyy-MM-ddTHH:mm:ss) |

## 에러 응답

### 400 Bad Request - 유효하지 않은 초대 링크

```json
{
  "isSuccess": false,
  "code": "TEAM4002",
  "message": "유효하지 않은 초대 링크입니다.",
  "result": null
}
```

### 400 Bad Request - 만료된 초대 코드

```json
{
  "isSuccess": false,
  "code": "TEAM4003",
  "message": "만료된 초대 링크입니다. 팀 관리자에게 새로운 초대 링크를 요청해주세요.",
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

### 404 Not Found - 팀 없음

```json
{
  "isSuccess": false,
  "code": "TEAM4041",
  "message": "존재하지 않는 팀입니다.",
  "result": null
}
```

### 409 Conflict - 이미 멤버

```json
{
  "isSuccess": false,
  "code": "TEAM4092",
  "message": "이미 해당 팀의 멤버입니다.",
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
| TEAM4002 | 400 | 유효하지 않은 초대 링크 | URL 형식 오류 또는 inviteCode 추출 실패 |
| TEAM4003 | 400 | 만료된 초대 코드 | 초대 코드 생성 후 7일이 경과하여 만료됨 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료 또는 잘못된 Bearer 토큰 |
| TEAM4041 | 404 | 존재하지 않는 팀 | 초대 링크의 inviteCode와 매칭되는 팀이 DB에 없음 |
| TEAM4092 | 409 | 이미 팀 멤버 | 이미 해당 팀에 가입된 사용자가 다시 가입 시도 |
| COMMON500 | 500 | 서버 내부 에러 | 팀 합류 처리 중 DB 연결 오류 등 |

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/v1/teams/join \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "inviteUrl": "https://service.com/invite/INV-A1B2-C3D4"
  }'
```
