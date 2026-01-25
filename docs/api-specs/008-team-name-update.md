# [API-008] PATCH /api/teams/{teamId}/name

팀 이름 변경 API

## 개요

기존 팀의 이름을 새로운 이름으로 변경합니다.

- 팀 관리자(Owner) 권한을 가진 사용자만 변경할 수 있습니다.
- 이름은 중복을 허용하지 않으며, 최대 20자 제한을 준수해야 합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

## 엔드포인트

```
PATCH /api/teams/{teamId}/name
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
| teamId | long | Yes | 이름을 변경할 팀의 고유 식별자 |

### Body

```json
{
  "name": "새로운 팀 이름"
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| name | string | Yes | 변경할 새로운 팀 이름 | 1~20자 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "팀 이름 변경에 성공하였습니다.",
  "result": {
    "teamId": 123,
    "teamName": "새로운 팀 이름",
    "updatedAt": "2026-01-24T15:30:00"
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| teamId | long | 팀 고유 ID |
| teamName | string | 변경된 팀 이름 |
| updatedAt | string | 수정 일시 (yyyy-MM-ddTHH:mm:ss) |

> **필드명 참고**: 요청 Body는 `name`, 응답은 `teamName`으로 다른 API와의 일관성을 유지합니다.

## 에러 응답

### 400 Bad Request - 팀 이름 길이 초과

```json
{
  "isSuccess": false,
  "code": "TEAM4001",
  "message": "팀 이름은 20자를 초과할 수 없습니다.",
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
  "message": "팀 이름을 변경할 권한이 없습니다.",
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

### 409 Conflict - 이름 중복

```json
{
  "isSuccess": false,
  "code": "TEAM4091",
  "message": "이미 사용 중인 팀 이름입니다.",
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

| Code | HTTP Status | Description |
|------|-------------|-------------|
| TEAM4001 | 400 | 글자 수 제한 유효성 검사 실패 |
| AUTH4001 | 401 | 토큰 누락, 만료 또는 잘못된 형식 |
| TEAM4031 | 403 | 팀 관리자가 아닌 유저가 수정 시도 |
| TEAM4041 | 404 | 유효하지 않은 teamId |
| TEAM4091 | 409 | 중복된 팀 이름으로 변경 시도 |
| COMMON500 | 500 | 이름 변경 처리 중 서버 에러 |

## 사용 예시

### cURL

```bash
curl -X PATCH https://api.example.com/api/teams/123/name \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "name": "새로운 팀 이름"
  }'
```
