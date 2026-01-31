# 🔐 Auth APIs

> 인증 관련 API 상세 명세

---

## 📍 Overview

```mermaid
flowchart LR
    subgraph auth["Auth APIs"]
        A1["API-001<br/>소셜 로그인"]
        A2["API-002<br/>회원가입"]
        A3["API-003<br/>토큰 갱신"]
        A4["API-004<br/>로그아웃"]
    end

    A1 -->|신규| A2
    A1 -->|기존| TOKEN["Tokens"]
    A2 --> TOKEN
    TOKEN -->|만료| A3
    TOKEN --> A4
```

---

## API-001 소셜 로그인

> `POST /api/v1/auth/social-login`

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant P as Provider

    C->>S: { provider, accessToken }
    S->>P: 토큰 검증
    P-->>S: 사용자 정보

    alt 신규 회원
        S-->>C: { isNewMember: true, signupToken }
    else 기존 회원
        S-->>C: { isNewMember: false, accessToken, refreshToken }
    end
```

### Request

```json
{
  "provider": "KAKAO",       // KAKAO | GOOGLE
  "accessToken": "소셜_토큰"
}
```

### Response

| 상황 | Code | Response |
|------|------|----------|
| 신규 | AUTH2001 | `isNewMember: true, signupToken` |
| 기존 | COMMON200 | `isNewMember: false, accessToken, refreshToken` |

### Errors

| Code | HTTP | 설명 |
|------|------|------|
| AUTH4002 | 401 | 유효하지 않은 소셜 토큰 |

→ [[apis/API-001 소셜 로그인|상세 문서]]

---

## API-002 회원가입

> `POST /api/v1/auth/signup`

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: { signupToken, nickname }
    S->>S: 토큰 검증

    alt 토큰 무효
        S-->>C: 401 AUTH4003
    end

    S->>DB: 회원 생성
    S-->>C: { accessToken, refreshToken }
```

### Request

```json
{
  "signupToken": "eyJhbG...",
  "nickname": "홍길동"
}
```

### Response

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "accessToken": "eyJhbG...",
    "refreshToken": "eyJhbG..."
  }
}
```

### Errors

| Code | HTTP | 설명 |
|------|------|------|
| AUTH4003 | 400 | 유효하지 않은 회원가입 토큰 |
| AUTH4091 | 409 | 이미 가입된 이메일 |

→ [[apis/API-002 회원가입|상세 문서]]

---

## API-003 토큰 리프레시

> `POST /api/v1/auth/token/refresh`

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: { refreshToken }
    S->>S: 토큰 검증
    S->>DB: 토큰 존재 확인

    alt 유효
        S-->>C: { accessToken }
    else 무효/로그아웃됨
        S-->>C: 401 Unauthorized
    end
```

### Request

```json
{
  "refreshToken": "eyJhbG..."
}
```

### Response

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "accessToken": "eyJhbG..."
  }
}
```

### Errors

| Code | HTTP | 설명 |
|------|------|------|
| AUTH4004 | 401 | 유효하지 않은 리프레시 토큰 |
| AUTH4005 | 401 | 로그아웃된 토큰 |

→ [[apis/API-003 토큰 리프레시|상세 문서]]

---

## API-004 로그아웃

> `POST /api/v1/auth/logout`

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: Authorization: Bearer {accessToken}
    C->>S: { refreshToken }

    S->>DB: DELETE refresh_token
    S-->>C: { message: "로그아웃 되었습니다" }
```

### Headers

```
Authorization: Bearer {accessToken}
```

### Request

```json
{
  "refreshToken": "eyJhbG..."
}
```

### Response

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "message": "로그아웃 되었습니다"
  }
}
```

→ [[apis/API-004 로그아웃|상세 문서]]

---

## 🎫 Token Summary

```mermaid
flowchart TB
    subgraph tokens["Token Types"]
        ACCESS["🟢 Access Token<br/>30분"]
        REFRESH["🔵 Refresh Token<br/>14일"]
        SIGNUP["🟡 Signup Token<br/>10분"]
    end

    subgraph claims["JWT Claims"]
        SUB["sub: user_id"]
        TYPE["token_type"]
        JTI["jti (refresh only)"]
        EMAIL["email (signup only)"]
    end

    tokens --- claims
```

| Token | TTL | 용도 | 특징 |
|-------|-----|------|------|
| Access | 30분 | API 인증 | `token_type: "access"` |
| Refresh | 14일 | 토큰 갱신 | `jti` 포함, DB 저장 |
| Signup | 10분 | 회원가입 | `email`, `provider` 포함 |

---

## 🚨 Error Codes

| Code | HTTP | 설명 | 대응 |
|------|------|------|------|
| AUTH2001 | 200 | 신규 회원 | 회원가입 진행 |
| AUTH4001 | 401 | 인증 실패 | 재로그인 |
| AUTH4002 | 401 | 무효한 소셜 토큰 | 소셜 재인증 |
| AUTH4003 | 400 | 무효한 회원가입 토큰 | 로그인 재시도 |
| AUTH4004 | 401 | 무효한 리프레시 토큰 | 재로그인 |
| AUTH4005 | 401 | 로그아웃된 토큰 | 재로그인 |
| AUTH4091 | 409 | 이미 가입된 이메일 | 로그인 시도 |

---

## 🔗 Related

- [[00-HOME|🏠 HOME]]
- [[02-Auth-Flow|🔐 Auth Flow]]
- [[05-API-Overview|🔌 API Overview]]

---

#auth #api #login #token
