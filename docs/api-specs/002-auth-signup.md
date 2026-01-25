# [API-002] POST /api/v1/auth/signup

회원가입 API (닉네임 등록)

## 개요

소셜 로그인 단계에서 획득한 이메일과 사용자가 입력한 닉네임을 전달하여 최종적으로 회원가입을 완료합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | 토큰 유효기간(TTL) 정보 추가 |

- 이 API는 **Access Token(JWT)**을 헤더로 전달받아 인증된 상태에서 호출되어야 합니다.

### 토큰 유효기간 (TTL)

| 토큰 타입 | 유효기간 | 설명 |
|----------|---------|------|
| accessToken | 30분 | API 요청 시 인증에 사용, 만료 시 refreshToken으로 재발급 필요 |
| refreshToken | 14일 | accessToken 재발급에 사용, 만료 시 재로그인 필요 |

## 엔드포인트

```
POST /api/v1/auth/signup
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
  "email": "user@example.com",
  "nickname": "제이슨"
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| email | string | Yes | 소셜 로그인 API에서 반환받은 이메일 | 이메일 형식 |
| nickname | string | Yes | 사용자가 설정할 서비스 닉네임 | 1~20자, 특수문자 제외 |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회원가입이 성공적으로 완료되었습니다.",
  "result": {
    "memberId": 505,
    "nickname": "제이슨",
    "accessToken": "service_access_token_xxx",
    "refreshToken": "service_refresh_token_xxx"
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| memberId | long | 생성된 회원의 고유 식별자 |
| nickname | string | 설정된 닉네임 |
| accessToken | string | 서비스 자체 Access Token (유효기간: 30분) |
| refreshToken | string | 서비스 자체 Refresh Token (유효기간: 14일) |

## 에러 응답

### 400 Bad Request - 유효성 검증 실패

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "닉네임은 1~20자 이내로 입력해야 합니다.",
  "result": null
}
```

### 409 Conflict - 닉네임 중복

```json
{
  "isSuccess": false,
  "code": "MEMBER4091",
  "message": "이미 사용 중인 닉네임입니다.",
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

### 404 Not Found - 유효하지 않은 회원가입 정보

```json
{
  "isSuccess": false,
  "code": "MEMBER4041",
  "message": "유효하지 않은 회원가입 정보입니다.",
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
| COMMON400 | 400 | 유효성 검증 실패 | 닉네임이 1~20자 범위를 벗어나거나 특수문자 포함 시 |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | 토큰 누락, 만료 또는 잘못된 Bearer 토큰 |
| MEMBER4041 | 404 | 유효하지 않은 회원가입 정보 | API-001에서 신규 회원으로 확인된 상태가 아닌 경우 |
| MEMBER4091 | 409 | 닉네임 중복 | 이미 사용 중인 닉네임으로 가입 시도 |
| COMMON500 | 500 | 서버 내부 에러 | 서버 로직 처리 중 예외 발생 |

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/v1/auth/signup \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "email": "user@example.com",
    "nickname": "제이슨"
  }'
```
