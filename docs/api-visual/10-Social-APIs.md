# 💬 Social APIs

> 소셜 기능 (좋아요, 댓글) API 상세 명세

---

## 📍 Overview

```mermaid
flowchart LR
    subgraph response["회고 답변"]
        RESPONSE["Response"]
    end

    subgraph social["소셜 기능"]
        LIKE["좋아요<br/>API-025"]
        COMMENT_LIST["댓글 조회<br/>API-026"]
        COMMENT_CREATE["댓글 작성<br/>API-027"]
    end

    RESPONSE --> LIKE
    RESPONSE --> COMMENT_LIST
    COMMENT_LIST --> COMMENT_CREATE
```

---

## 🔄 상호작용 흐름

```mermaid
sequenceDiagram
    participant U1 as 사용자 A
    participant S as Server
    participant U2 as 사용자 B

    Note over U1,U2: 답변 확인 후 상호작용

    U1->>S: 좋아요 (API-025)
    U1->>S: 댓글 작성 (API-027)

    U2->>S: 댓글 조회 (API-026)
    U2->>S: 좋아요 (API-025)
    U2->>S: 댓글 작성 (API-027)
```

---

## API-025 좋아요 토글

> `POST /api/v1/responses/:id/likes` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: POST /responses/1/likes

    S->>DB: SELECT response_like

    alt 좋아요 있음
        S->>DB: DELETE response_like
        S-->>C: { isLiked: false, likeCount: 4 }
    else 좋아요 없음
        S->>DB: INSERT response_like
        S-->>C: { isLiked: true, likeCount: 6 }
    end
```

### Response

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "isLiked": true,
    "likeCount": 6
  }
}
```

### 동작 방식

```mermaid
flowchart LR
    subgraph toggle["토글 동작"]
        CHECK{"좋아요<br/>존재?"}
        ADD["추가"]
        REMOVE["제거"]
    end

    CHECK -->|No| ADD -->|isLiked: true| RESULT["결과"]
    CHECK -->|Yes| REMOVE -->|isLiked: false| RESULT
```

### Errors

| Code | HTTP | 설명 |
|------|------|------|
| RES4041 | 404 | 답변을 찾을 수 없음 |

→ [[apis/API-026 좋아요 토글|상세 문서]]

---

## API-026 댓글 조회

> `GET /api/v1/responses/:id/comments` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: GET /responses/1/comments
    S->>DB: SELECT response_comments<br/>JOIN member
    DB-->>S: comments with author
    S-->>C: { comments: [...] }
```

### Response

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "comments": [
      {
        "commentId": 1,
        "content": "좋은 의견이네요!",
        "author": {
          "memberId": 2,
          "nickname": "김철수"
        },
        "createdAt": "2024-01-15T14:30:00Z",
        "isOwner": false
      },
      {
        "commentId": 2,
        "content": "저도 같은 생각입니다",
        "author": {
          "memberId": 1,
          "nickname": "홍길동"
        },
        "createdAt": "2024-01-15T14:35:00Z",
        "isOwner": true
      }
    ]
  }
}
```

### Response Fields

| Field | Type | 설명 |
|-------|------|------|
| `commentId` | number | 댓글 ID |
| `content` | string | 댓글 내용 |
| `author` | object | 작성자 정보 |
| `createdAt` | string | 작성 시간 |
| `isOwner` | boolean | 본인 작성 여부 |

→ [[apis/API-027 댓글 조회|상세 문서]]

---

## API-027 댓글 작성

> `POST /api/v1/responses/:id/comments` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: POST /responses/1/comments
    Note right of C: { content: "좋은 의견이네요!" }

    S->>DB: Check response exists

    alt 답변 없음
        S-->>C: 404 Not Found
    end

    S->>DB: INSERT response_comment
    S-->>C: 201 Created
```

### Request

```json
{
  "content": "좋은 의견이네요!"
}
```

### Validation

| Field | 조건 |
|-------|------|
| `content` | 필수, 1자 이상 |

### Response

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "commentId": 3,
    "content": "좋은 의견이네요!",
    "createdAt": "2024-01-15T15:00:00Z"
  }
}
```

### Errors

| Code | HTTP | 설명 |
|------|------|------|
| RES4041 | 404 | 답변을 찾을 수 없음 |
| COMMON400 | 400 | 내용이 비어있음 |

→ [[apis/API-028 댓글 작성|상세 문서]]

---

## 📊 데이터 모델

```mermaid
erDiagram
    RESPONSE ||--o{ RESPONSE_LIKE : has
    RESPONSE ||--o{ RESPONSE_COMMENT : has
    MEMBER ||--o{ RESPONSE_LIKE : creates
    MEMBER ||--o{ RESPONSE_COMMENT : writes

    RESPONSE {
        bigint response_id PK
        bigint retrospect_id FK
        string question
        text content
    }

    RESPONSE_LIKE {
        bigint id PK
        bigint response_id FK
        bigint member_id FK
        datetime created_at
    }

    RESPONSE_COMMENT {
        bigint id PK
        bigint response_id FK
        bigint member_id FK
        text content
        datetime created_at
    }
```

---

## 🎯 사용 시나리오

```mermaid
journey
    title 소셜 상호작용 시나리오
    section 답변 확인
      카테고리별 조회: 5: User
      답변 읽기: 4: User
    section 상호작용
      좋아요 추가: 5: User
      댓글 작성: 4: User
      다른 댓글 확인: 3: User
    section 반복
      다른 답변 확인: 4: User
      좋아요/댓글: 5: User
```

---

## 🔐 권한

| API | 요구 권한 | 설명 |
|-----|----------|------|
| 좋아요 토글 | 🔐 로그인 | 누구나 가능 |
| 댓글 조회 | 🔐 로그인 | 누구나 가능 |
| 댓글 작성 | 🔐 로그인 | 누구나 가능 |

> [!note] 회고방 멤버 제한
> 소셜 기능은 해당 회고방의 멤버만 사용할 수 있습니다.

---

## 🚨 Error Codes

| Code | HTTP | 설명 |
|------|------|------|
| RES4041 | 404 | 답변을 찾을 수 없음 |
| COMMON400 | 400 | 잘못된 요청 (빈 내용) |
| COMMON401 | 401 | 인증 필요 |

---

## 🔗 Related

- [[00-HOME|🏠 HOME]]
- [[09-Retrospect-APIs|📝 Retrospect APIs]]
- [[05-API-Overview|🔌 API Overview]]

---

#social #like #comment #api
