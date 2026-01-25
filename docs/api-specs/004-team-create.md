# [API-004] POST /api/v1/teams

팀 생성 API

## 개요

새로운 팀을 생성합니다.

- 팀을 생성한 사용자는 해당 팀의 **관리자(Owner)** 권한을 자동으로 부여받습니다.
- 생성과 동시에 다른 사용자를 초대할 수 있는 고유한 **초대 코드(`inviteCode`)**가 생성됩니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

## 엔드포인트

```
POST /api/v1/teams
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
  "teamName": "코드 마스터즈",
  "description": "우리 팀의 성장을 위한 회고 모임입니다."
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| teamName | string | Yes | 팀 이름 | 최대 20자 |
| description | string | No | 팀 한 줄 소개 | 최대 50자 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "팀이 성공적으로 생성되었습니다.",
  "result": {
    "teamId": 789,
    "teamName": "코드 마스터즈",
    "inviteCode": "INV-A1B2-C3D4"
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| teamId | long | 생성된 팀 고유 ID |
| teamName | string | 생성된 팀 이름 |
| inviteCode | string | 팀원 초대를 위한 고유 코드 (형식: `INV-XXXX-XXXX`, 8자리 영숫자). 생성 후 7일간 유효하며, 만료 후 재발급 가능합니다. |

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

### 409 Conflict - 팀 이름 중복

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

| Code | HTTP Status | Description | 발생 조건 |
|------|-------------|-------------|-----------|
| TEAM4001 | 400 | 팀 이름 길이 초과 | teamName이 20자를 초과한 경우 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료 또는 잘못된 Bearer 토큰 |
| TEAM4091 | 409 | 팀 이름 중복 | 이미 사용 중인 teamName으로 생성 시도 |
| COMMON500 | 500 | 서버 내부 에러 | 팀 생성 과정 중 DB 연결 오류 등 |

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/v1/teams \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "teamName": "코드 마스터즈",
    "description": "우리 팀의 성장을 위한 회고 모임입니다."
  }'
```
