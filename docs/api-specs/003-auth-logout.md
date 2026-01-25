# [API-003] POST /api/v1/auth/logout

로그아웃 API

## 개요

현재 사용자의 로그아웃을 처리합니다.

## 버전

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0.0 | 2025-01-25 | 최초 작성 |

- 서버에 저장된 **Refresh Token**을 삭제하거나 무효화하여 보안을 유지합니다.
- 로그아웃 성공 후 해당 Refresh Token을 이용한 Access Token 재발급은 불가능해집니다.

## 엔드포인트

```
POST /api/v1/auth/logout
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
  "refreshToken": "service_refresh_token_xxx"
}
```

### 필드 설명

| Field | Type | Required | Description | Validation |
|-------|------|----------|-------------|------------|
| refreshToken | string | Yes | 서버에서 무효화 처리할 리프레시 토큰 | - |

> **참고**: `refreshToken`을 요청 Body에 포함하는 이유는 다중 기기 로그인 지원을 위해서입니다. 사용자가 여러 기기에서 로그인한 경우, 특정 기기만 로그아웃할 수 있도록 해당 기기의 Refresh Token을 명시적으로 무효화합니다.

## Response

### 성공 (200 OK)

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "로그아웃이 성공적으로 처리되었습니다.",
  "result": null
}
```

### 응답 필드

| Field | Type | Description |
|-------|------|-------------|
| result | null | 로그아웃 성공 시 별도 데이터 없음 |

## 에러 응답

### 400 Bad Request - 유효하지 않은 토큰

```json
{
  "isSuccess": false,
  "code": "AUTH4003",
  "message": "이미 로그아웃되었거나 유효하지 않은 토큰입니다.",
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
| AUTH4003 | 400 | 유효하지 않은 토큰 | 이미 로그아웃되었거나 존재하지 않는 refreshToken |
| AUTH4001 | 401 | 인증 정보가 유효하지 않음 | accessToken 누락, 만료 또는 잘못된 Bearer 토큰 |
| COMMON500 | 500 | 서버 내부 에러 | 토큰 삭제 처리 중 DB 연결 오류 |

## 사용 예시

### cURL

```bash
curl -X POST https://api.example.com/api/v1/auth/logout \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {accessToken}" \
  -d '{
    "refreshToken": "service_refresh_token_xxx"
  }'
```
