# 🔐 Authentication Flow

> 소셜 로그인부터 로그아웃까지 전체 인증 플로우

---

## 📍 Overview

```mermaid
flowchart LR
    subgraph entry["Entry Points"]
        GOOGLE["🔵 Google"]
        KAKAO["🟡 Kakao"]
    end

    subgraph auth["Auth Flow"]
        LOGIN["소셜 로그인"]
        SIGNUP["회원가입"]
        REFRESH["토큰 갱신"]
        LOGOUT["로그아웃"]
    end

    subgraph tokens["Tokens"]
        AT["Access Token"]
        RT["Refresh Token"]
        ST["Signup Token"]
    end

    entry --> LOGIN
    LOGIN -->|신규| ST
    LOGIN -->|기존| AT & RT
    ST --> SIGNUP --> AT & RT
    RT --> REFRESH --> AT
    AT --> LOGOUT
```

---

## 1️⃣ 소셜 로그인 (API-001)

```mermaid
sequenceDiagram
    autonumber
    participant C as 📱 Client
    participant S as 🦀 Server
    participant G as 🌐 Google/Kakao
    participant DB as 💾 Database

    Note over C,DB: 소셜 로그인 요청
    C->>S: POST /api/v1/auth/social-login
    Note right of C: { provider, accessToken }

    S->>G: 토큰 검증 요청
    G-->>S: 사용자 정보 (email)

    S->>DB: SELECT member WHERE email

    alt 🆕 신규 회원
        DB-->>S: Not Found
        S->>S: Generate Signup Token
        Note over S: Claims: { email, provider, token_type: "signup" }
        S-->>C: 200 OK
        Note left of S: { isNewMember: true, email, signupToken }
    else 🔄 기존 회원
        DB-->>S: Member Found
        S->>S: Generate Access Token
        S->>S: Generate Refresh Token
        S->>DB: INSERT refresh_token
        S-->>C: 200 OK
        Note left of S: { isNewMember: false, accessToken, refreshToken }
    end
```

### Request / Response

```json
// Request
{
  "provider": "KAKAO",      // KAKAO | GOOGLE
  "accessToken": "소셜_액세스_토큰"
}

// Response (신규 회원)
{
  "isSuccess": true,
  "code": "AUTH2001",
  "result": {
    "isNewMember": true,
    "email": "user@example.com",
    "signupToken": "eyJhbG..."
  }
}

// Response (기존 회원)
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "isNewMember": false,
    "accessToken": "eyJhbG...",
    "refreshToken": "eyJhbG..."
  }
}
```

---

## 2️⃣ 회원가입 (API-002)

```mermaid
sequenceDiagram
    autonumber
    participant C as 📱 Client
    participant S as 🦀 Server
    participant DB as 💾 Database

    Note over C,DB: 신규 회원 가입 (signupToken 필요)
    C->>S: POST /api/v1/auth/signup
    Note right of C: { signupToken, nickname }

    S->>S: Validate Signup Token
    Note over S: token_type == "signup" 확인

    alt ❌ 토큰 만료/무효
        S-->>C: 401 Unauthorized
        Note left of S: AUTH4003
    end

    S->>DB: SELECT member WHERE email

    alt ⚠️ 이미 가입됨
        DB-->>S: Member Exists
        S-->>C: 409 Conflict
        Note left of S: AUTH4091
    end

    S->>DB: INSERT member
    DB-->>S: OK

    S->>S: Generate Access Token
    S->>S: Generate Refresh Token
    S->>DB: INSERT refresh_token

    S-->>C: 200 OK
    Note left of S: { accessToken, refreshToken }
```

### Request / Response

```json
// Request
{
  "signupToken": "eyJhbG...",
  "nickname": "홍길동"
}

// Response
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "accessToken": "eyJhbG...",
    "refreshToken": "eyJhbG..."
  }
}
```

---

## 3️⃣ 토큰 갱신 (API-003)

```mermaid
sequenceDiagram
    autonumber
    participant C as 📱 Client
    participant S as 🦀 Server
    participant DB as 💾 Database

    Note over C,DB: Access Token 갱신
    C->>S: POST /api/v1/auth/token/refresh
    Note right of C: { refreshToken }

    S->>S: Validate Refresh Token
    Note over S: token_type == "refresh" 확인

    alt ❌ 토큰 무효
        S-->>C: 401 Unauthorized
        Note left of S: AUTH4004
    end

    S->>DB: SELECT refresh_token WHERE token

    alt ❌ DB에 없음 (로그아웃됨)
        S-->>C: 401 Unauthorized
        Note left of S: AUTH4005
    end

    S->>S: Generate New Access Token
    S-->>C: 200 OK
    Note left of S: { accessToken }
```

### Request / Response

```json
// Request
{
  "refreshToken": "eyJhbG..."
}

// Response
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "accessToken": "eyJhbG..."
  }
}
```

---

## 4️⃣ 로그아웃 (API-004)

```mermaid
sequenceDiagram
    autonumber
    participant C as 📱 Client
    participant S as 🦀 Server
    participant DB as 💾 Database

    Note over C,DB: 로그아웃 (refreshToken 무효화)
    C->>S: POST /api/v1/auth/logout
    Note right of C: Authorization: Bearer {accessToken}
    Note right of C: { refreshToken }

    S->>S: Validate Access Token

    S->>DB: DELETE refresh_token WHERE token
    DB-->>S: OK

    S-->>C: 200 OK
    Note left of S: { message: "로그아웃 되었습니다" }
```

### Request / Response

```json
// Request
{
  "refreshToken": "eyJhbG..."
}

// Response
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "message": "로그아웃 되었습니다"
  }
}
```

---

## 🎫 Token Comparison

```mermaid
flowchart TB
    subgraph access["🟢 Access Token"]
        AT_TTL["TTL: 30분"]
        AT_USE["용도: API 인증"]
        AT_STORE["저장: 메모리"]
    end

    subgraph refresh["🔵 Refresh Token"]
        RT_TTL["TTL: 14일"]
        RT_USE["용도: Access 갱신"]
        RT_STORE["저장: DB + 클라이언트"]
        RT_JTI["jti: 고유 ID"]
    end

    subgraph signup["🟡 Signup Token"]
        ST_TTL["TTL: 10분"]
        ST_USE["용도: 회원가입"]
        ST_DATA["포함: email, provider"]
    end
```

| 토큰 | 유효기간 | 용도 | 특징 |
|------|---------|------|------|
| **Access** | 30분 | API 인증 | `token_type: "access"` |
| **Refresh** | 14일 | 토큰 갱신 | `jti` 포함, DB 저장 |
| **Signup** | 10분 | 회원가입 | `email`, `provider` 포함 |

---

## 🚨 Error Codes

| Code | HTTP | 상황 | 대응 |
|------|------|------|------|
| AUTH4001 | 401 | 인증 실패 | 재로그인 |
| AUTH4002 | 401 | 무효한 소셜 토큰 | 소셜 재인증 |
| AUTH4003 | 400 | 무효한 회원가입 토큰 | 로그인 재시도 |
| AUTH4004 | 401 | 무효한 리프레시 토큰 | 재로그인 |
| AUTH4005 | 401 | 로그아웃된 토큰 | 재로그인 |
| AUTH4091 | 409 | 이미 가입된 이메일 | 로그인 시도 |

---

## 🔄 Token Lifecycle

```mermaid
stateDiagram-v2
    [*] --> SocialLogin: 앱 시작

    SocialLogin --> NewUser: 신규 회원
    SocialLogin --> HasTokens: 기존 회원

    NewUser --> Signup: signupToken
    Signup --> HasTokens: 가입 완료

    state HasTokens {
        [*] --> AccessValid
        AccessValid --> AccessExpired: 30분 경과
        AccessExpired --> AccessValid: Refresh 성공
        AccessExpired --> [*]: Refresh 실패
    }

    HasTokens --> Logout: 로그아웃
    Logout --> [*]
```

---

## 🔗 Related APIs

- [[apis/API-001 소셜 로그인|API-001 소셜 로그인]]
- [[apis/API-002 회원가입|API-002 회원가입]]
- [[apis/API-003 토큰 리프레시|API-003 토큰 리프레시]]
- [[apis/API-004 로그아웃|API-004 로그아웃]]

---

## 🔗 Navigation

- [[00-HOME|🏠 HOME]]
- [[01-Architecture|🏗️ Architecture]]
- [[03-Retrospect-Flow|📝 Retrospect Flow]] →

---

#auth #jwt #token #login #flow
