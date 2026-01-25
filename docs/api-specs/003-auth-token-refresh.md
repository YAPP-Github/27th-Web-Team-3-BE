# [API-003] POST /api/v1/auth/token/refresh

토큰 갱신 API

## 개요

만료된 Access Token을 Refresh Token을 이용하여 재발급합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

- Access Token 만료 시 재로그인 없이 새로운 토큰을 발급받을 수 있습니다.
- Refresh Token도 함께 갱신하여 보안을 강화합니다 (Refresh Token Rotation).

### 토큰 유효기간 (TTL)

| 토큰 타입 | 유효기간 | 설명 |
|----------|---------|------|
| accessToken | 30분 | API 요청 시 인증에 사용, 만료 시 이 API로 재발급 |
| refreshToken | 14일 | accessToken 재발급에 사용, 갱신 시 새로운 refreshToken도 함께 발급 |

## 엔드포인트

```
POST /api/v1/auth/token/refresh
```

## 인증

- Bearer 토큰 인증 불필요 (Refresh Token을 Body에 포함하여 전송)

## Request

### Headers

| Header | Value | Required |
|--------|-------|----------|
| Content-Type | application/json | Yes |

### Body

```json
{
  "refreshToken": "service_refresh_token_xxx"
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| refreshToken | string | Yes | 로그인 또는 회원가입 시 발급받은 유효한 Refresh Token | - |

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "토큰이 성공적으로 갱신되었습니다.",
  "result": {
    "accessToken": "new_service_access_token_xxx",
    "refreshToken": "new_service_refresh_token_xxx"
  }
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| accessToken | string | 새로 발급된 Access Token (유효기간: 30분) |
| refreshToken | string | 새로 발급된 Refresh Token (유효기간: 14일) |

> **참고**: Refresh Token Rotation 정책에 따라 기존 Refresh Token은 무효화되고 새로운 Refresh Token이 발급됩니다. 클라이언트는 반드시 새로 받은 Refresh Token을 저장해야 합니다.

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

### 401 Unauthorized - 유효하지 않은 Refresh Token

```json
{
  "isSuccess": false,
  "code": "AUTH4004",
  "message": "유효하지 않거나 만료된 Refresh Token입니다.",
  "result": null
}
```

### 401 Unauthorized - 로그아웃된 토큰

```json
{
  "isSuccess": false,
  "code": "AUTH4005",
  "message": "로그아웃 처리된 토큰입니다. 다시 로그인해 주세요.",
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
| COMMON400 | 400 | 필수 파라미터 누락 | refreshToken 필드가 누락된 경우 |
| AUTH4004 | 401 | 유효하지 않거나 만료된 Refresh Token | refreshToken이 14일 경과하여 만료되었거나 형식이 올바르지 않은 경우 |
| AUTH4005 | 401 | 로그아웃 처리된 토큰 | API-004(로그아웃)를 통해 무효화된 refreshToken을 사용한 경우 |
| COMMON500 | 500 | 서버 내부 에러 | 토큰 발급 처리 중 서버 오류 발생 |

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/v1/auth/token/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refreshToken": "service_refresh_token_xxx"
  }'
```

## 토큰 갱신 흐름

```
1. 클라이언트가 API 요청 시 Access Token 만료 감지 (401 응답)
   ↓
2. 저장된 Refresh Token으로 /api/v1/auth/token/refresh 호출
   ↓
3. 새로운 Access Token + Refresh Token 발급
   ↓
4. 새 토큰들을 저장하고 원래 API 재시도
   ↓
5. Refresh Token마저 만료된 경우 (14일 경과) → 재로그인 필요
```
