# [API-001] POST /api/v1/auth/social-login

소셜 로그인 API

## 개요

구글/카카오 Access Token을 전달받아 사용자 식별 정보를 확인합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |
| 1.1.0 | 2025-01-25 | 토큰 유효기간(TTL) 정보 추가 |
| 1.2.0 | 2026-01-25 | 신규 회원 응답에 signupToken 추가 |

- **기존 회원**: 서비스 자체 토큰(`accessToken`, `refreshToken`)을 즉시 발급합니다.
- **신규 회원**: `isNewMember: true`와 함께 이메일을 반환하여 추가 정보 입력(회원가입) 단계로 유도합니다.

### 토큰 유효기간 (TTL)

| 토큰 타입 | 유효기간 | 설명 |
|----------|---------|------|
| accessToken | 30분 | API 요청 시 인증에 사용, 만료 시 refreshToken으로 재발급 필요 |
| refreshToken | 14일 | accessToken 재발급에 사용, 만료 시 재로그인 필요 |
| signupToken | 10분 | 신규 회원의 회원가입 API 호출 시 인증에 사용 |

## 엔드포인트

```
POST /api/v1/auth/social-login
```

## 인증

- Bearer 토큰 인증 불필요 (클라이언트에서 발급받은 소셜 토큰으로 사용자 신원 확인)

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Content-Type | application/json | Yes |

### Body

```json
{
  "provider": "KAKAO",
  "accessToken": "sample_social_token_123"
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| provider | string (Enum) | Yes | 소셜 서비스 구분 | GOOGLE, KAKAO |
| accessToken | string | Yes | 소셜 서비스에서 발급받은 Access Token | - |

## Response

### 성공 - 기존 회원 로그인 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "로그인에 성공하였습니다.",
  "result": {
    "isNewMember": false,
    "accessToken": "service_access_token_xxx",
    "refreshToken": "refresh_token_token_xxx"
  }
}
```

### 성공 - 신규 회원 (200 OK)

```json
{
  "isSuccess": true,
  "code": "AUTH2001",
  "message": "신규 회원입니다. 가입 절차를 진행해 주세요.",
  "result": {
    "isNewMember": true,
    "email": "user@example.com",
    "signupToken": "signup_token_xxx"
  }
}
```

### 응답 필드 (기존 회원)

| Field | Type | Description |
|-------|------|-------------|
| isNewMember | boolean | false (기존 회원) |
| accessToken | string | 서비스 자체 발급 Access Token (유효기간: 30분) |
| refreshToken | string | 서비스 자체 발급 Refresh Token (유효기간: 14일) |

### 응답 필드 (신규 회원)

| Field | Type | Description |
|-------|------|-------------|
| isNewMember | boolean | true (신규 회원) |
| email | string | 소셜 계정에서 추출한 이메일 |
| signupToken | string | 회원가입 API 호출 시 사용할 임시 토큰 (유효기간: 10분) |

## 에러 응답

### 400 Bad Request - 필수 파라미터 누락

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "필수 파라미터가 누락되었습니다.",
  "result": null
}
```

### 401 Unauthorized - 유효하지 않은 소셜 토큰

```json
{
  "isSuccess": false,
  "code": "AUTH4002",
  "message": "유효하지 않은 소셜 토큰입니다.",
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
| AUTH2001 | 200 | 신규 회원 - 가입 절차 필요 (에러가 아닌 정상 분기 응답) | 소셜 계정 이메일로 등록된 기존 회원이 없는 경우 |
| COMMON400 | 400 | 필수 파라미터 누락 | provider 또는 accessToken 필드가 누락된 경우 |
| AUTH4002 | 401 | 유효하지 않은 소셜 토큰 | 구글/카카오 측에서 거부된 토큰 (만료, 변조 등) |
| COMMON500 | 500 | 서버 내부 에러 | 소셜 API 통신 중 서버 오류 발생 |

> **참고**: `AUTH2001`은 에러가 아닌 "추가 정보 입력 필요" 상태를 나타내는 정상 응답입니다. 클라이언트는 이 코드를 받으면 회원가입 화면으로 이동해야 합니다.

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/v1/auth/social-login \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "KAKAO",
    "accessToken": "sample_social_token_123"
  }'
```
