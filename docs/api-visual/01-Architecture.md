# 🏗️ System Architecture

> 회고록 AI 서비스 백엔드 전체 아키텍처

---

## 📐 High-Level Architecture

```mermaid
flowchart TB
    subgraph clients["🖥️ Clients"]
        WEB["Web App"]
        MOBILE["Mobile App"]
    end

    subgraph server["🦀 Rust Backend"]
        subgraph middleware["Middleware Layer"]
            CORS["CORS"]
            TRACE["Tracing"]
            AUTH_MW["Auth Extractor"]
        end

        subgraph handlers["Handler Layer"]
            AUTH_H["Auth Handler"]
            RETRO_H["Retrospect Handler"]
            MEMBER_H["Member Handler"]
        end

        subgraph services["Service Layer"]
            AUTH_S["Auth Service"]
            RETRO_S["Retrospect Service"]
            MEMBER_S["Member Service"]
            AI_S["AI Service"]
        end

        subgraph utils["Utils"]
            JWT["JWT Utils"]
            RESPONSE["Response Utils"]
            ERROR["Error Handler"]
        end
    end

    subgraph external["🌐 External Services"]
        GOOGLE["Google OAuth"]
        KAKAO["Kakao OAuth"]
        OPENAI["OpenAI API"]
    end

    subgraph storage["💾 Storage"]
        MYSQL[("MySQL")]
    end

    clients --> middleware
    middleware --> handlers
    handlers --> services
    services --> utils
    services --> external
    services --> storage
```

---

## 🗂️ Layer Architecture

```mermaid
flowchart LR
    subgraph presentation["🎨 Presentation Layer"]
        direction TB
        H1["handler.rs"]
        DTO["dto.rs"]
    end

    subgraph business["⚙️ Business Layer"]
        direction TB
        S1["service.rs"]
        PROMPT["prompt.rs"]
    end

    subgraph data["📦 Data Layer"]
        direction TB
        E1["entity/"]
        DB["database.rs"]
    end

    subgraph shared["🔧 Shared"]
        direction TB
        ERR["error.rs"]
        RESP["response.rs"]
        AUTH["auth.rs"]
        JWT["jwt.rs"]
    end

    presentation --> business
    business --> data
    presentation -.-> shared
    business -.-> shared
    data -.-> shared
```

---

## 📁 프로젝트 구조

```
codes/server/src/
├── main.rs                 # 🚀 Entry Point & Router
├── lib.rs                  # 📚 Public API
├── state.rs                # 🔄 AppState
│
├── config/                 # ⚙️ Configuration
│   ├── mod.rs
│   ├── app_config.rs       # 환경변수 설정
│   └── database.rs         # DB 연결 & 스키마
│
├── utils/                  # 🔧 Utilities
│   ├── mod.rs
│   ├── error.rs            # AppError
│   ├── response.rs         # BaseResponse
│   ├── auth.rs             # AuthUser Extractor
│   └── jwt.rs              # JWT 생성/검증
│
└── domain/                 # 📦 Domains
    ├── auth/               # 🔐 인증
    │   ├── handler.rs
    │   ├── service.rs
    │   └── dto.rs
    │
    ├── member/             # 👤 회원
    │   ├── handler.rs
    │   ├── service.rs
    │   ├── dto.rs
    │   └── entity/
    │       ├── member.rs
    │       ├── refresh_token.rs
    │       ├── member_retro.rs
    │       └── member_retro_room.rs
    │
    ├── retrospect/         # 📝 회고
    │   ├── handler.rs
    │   ├── service.rs
    │   ├── dto.rs
    │   └── entity/
    │       ├── retro_room.rs
    │       ├── retrospect.rs
    │       ├── response.rs
    │       ├── response_comment.rs
    │       ├── response_like.rs
    │       └── retro_reference.rs
    │
    └── ai/                 # 🤖 AI
        ├── service.rs
        └── prompt.rs
```

---

## 🔄 Request Flow

```mermaid
sequenceDiagram
    autonumber
    participant C as Client
    participant MW as Middleware
    participant H as Handler
    participant S as Service
    participant DB as Database
    participant EXT as External API

    C->>MW: HTTP Request
    Note over MW: CORS Check
    Note over MW: Tracing Start
    MW->>MW: Auth Token Extract

    alt Token Required
        MW->>MW: JWT Validate
        alt Invalid Token
            MW-->>C: 401 Unauthorized
        end
    end

    MW->>H: Pass Request
    H->>H: Validate Input

    alt Validation Failed
        H-->>C: 400 Bad Request
    end

    H->>S: Call Service
    S->>DB: Query/Mutation
    DB-->>S: Result

    opt External API Call
        S->>EXT: API Request
        EXT-->>S: Response
    end

    S-->>H: Service Result
    H-->>C: JSON Response
```

---

## 🔐 Authentication Architecture

```mermaid
flowchart TB
    subgraph tokens["🎫 Token Types"]
        ACCESS["Access Token<br/>30분 유효"]
        REFRESH["Refresh Token<br/>14일 유효"]
        SIGNUP["Signup Token<br/>10분 유효"]
    end

    subgraph claims["📋 JWT Claims"]
        direction LR
        SUB["sub: user_id"]
        IAT["iat: issued_at"]
        EXP["exp: expiration"]
        JTI["jti: token_id"]
        TYPE["token_type"]
    end

    subgraph flow["🔄 Token Flow"]
        direction TB
        LOGIN["소셜 로그인"]

        LOGIN -->|신규회원| SIGNUP
        LOGIN -->|기존회원| ACCESS
        LOGIN -->|기존회원| REFRESH

        SIGNUP -->|회원가입| ACCESS
        SIGNUP -->|회원가입| REFRESH

        REFRESH -->|갱신| ACCESS
    end

    tokens --> claims
```

---

## ⚙️ AppState

```mermaid
classDiagram
    class AppState {
        +DatabaseConnection db
        +AppConfig config
        +AiService ai_service
    }

    class AppConfig {
        +u16 server_port
        +String jwt_secret
        +i64 jwt_expiration
        +i64 refresh_token_expiration
        +i64 signup_token_expiration
        +String google_client_id
        +String kakao_client_id
        +String openai_api_key
    }

    class AiService {
        +Client client
        +analyze_retrospective()
    }

    AppState --> AppConfig
    AppState --> AiService
```

---

## 🛡️ Error Handling

```mermaid
flowchart LR
    subgraph errors["Error Types"]
        E400["BadRequest<br/>400"]
        E401["Unauthorized<br/>401"]
        E403["Forbidden<br/>403"]
        E404["NotFound<br/>404"]
        E409["Conflict<br/>409"]
        E500["Internal<br/>500"]
    end

    subgraph codes["Error Codes"]
        COMMON["COMMON4xx"]
        AUTH["AUTH4xxx"]
        RETRO["RETRO4xxx"]
        AI["AI4xxx/5xxx"]
    end

    subgraph response["Response Format"]
        JSON["
        {
          isSuccess: false,
          code: 'XXX',
          message: '...',
          result: null
        }
        "]
    end

    errors --> codes --> response
```

---

## 📦 Dependencies

| Category | Library | Version | 용도 |
|----------|---------|---------|------|
| **Web** | axum | 0.7 | Web Framework |
| **Async** | tokio | 1.0 | Runtime |
| **ORM** | sea-orm | 1.1 | Database |
| **Auth** | jsonwebtoken | 10.2 | JWT |
| **AI** | async-openai | 0.25 | OpenAI |
| **Docs** | utoipa | 4.0 | OpenAPI |
| **Log** | tracing | 0.1 | Logging |
| **Validate** | validator | 0.18 | Input |

---

## 🔗 Related

- [[00-HOME|🏠 HOME]]
- [[04-Entity-Diagram|📊 Entity Diagram]]
- [[05-API-Overview|🔌 API Overview]]

---

#architecture #system #overview
