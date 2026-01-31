# 📊 Entity Relationship Diagram

> 데이터베이스 스키마 및 엔티티 관계

---

## 📍 Overview

```mermaid
erDiagram
    MEMBER ||--o{ REFRESH_TOKEN : has
    MEMBER ||--o{ MEMBER_RETRO_ROOM : joins
    MEMBER ||--o{ MEMBER_RETRO : participates
    MEMBER ||--o{ RESPONSE_COMMENT : writes
    MEMBER ||--o{ RESPONSE_LIKE : likes

    RETRO_ROOM ||--o{ RETROSPECT : contains
    RETRO_ROOM ||--o{ MEMBER_RETRO_ROOM : has_members

    RETROSPECT ||--o{ RESPONSE : has
    RETROSPECT ||--o{ RETRO_REFERENCE : has
    RETROSPECT ||--o{ MEMBER_RETRO : has_participants

    RESPONSE ||--o{ RESPONSE_COMMENT : has
    RESPONSE ||--o{ RESPONSE_LIKE : has
```

---

## 🧩 Entity Details

### 👤 Member (회원)

```mermaid
erDiagram
    MEMBER {
        bigint member_id PK
        string email UK "고유 이메일"
        string nickname "닉네임"
        string social_type "GOOGLE | KAKAO"
        int insight_count "AI 분석 횟수"
        datetime created_at
        datetime updated_at
    }
```

| Field | Type | 설명 |
|-------|------|------|
| `member_id` | BIGINT | PK, Auto Increment |
| `email` | VARCHAR | Unique, 소셜 이메일 |
| `nickname` | VARCHAR | 사용자 닉네임 |
| `social_type` | ENUM | GOOGLE, KAKAO |
| `insight_count` | INT | 월간 AI 분석 횟수 |

---

### 🎫 Refresh Token (토큰)

```mermaid
erDiagram
    REFRESH_TOKEN {
        bigint id PK
        bigint member_id FK
        string token UK "JWT Token"
        datetime expires_at
        datetime created_at
    }

    MEMBER ||--o{ REFRESH_TOKEN : has
```

> [!note] 토큰 관리
> - 로그아웃 시 토큰 삭제
> - 토큰 갱신 시 기존 토큰 유지

---

### 🏠 Retro Room (회고방)

```mermaid
erDiagram
    RETRO_ROOM {
        bigint retro_room_id PK
        string title UK "회고방 이름"
        string description "설명"
        string invitation_url UK "INV-XXXX-XXXX"
        datetime created_at
        datetime updated_at
    }
```

| Field | Type | 설명 |
|-------|------|------|
| `retro_room_id` | BIGINT | PK |
| `title` | VARCHAR | Unique, 회고방 이름 |
| `invitation_url` | VARCHAR | Unique, 초대 코드 |
| `description` | TEXT | 회고방 설명 |

---

### 👥 Member Retro Room (회고방 멤버십)

```mermaid
erDiagram
    MEMBER_RETRO_ROOM {
        bigint id PK
        bigint member_id FK
        bigint retro_room_id FK
        string role "OWNER | MEMBER"
        int order_index "정렬 순서"
        datetime created_at
    }

    MEMBER ||--o{ MEMBER_RETRO_ROOM : joins
    RETRO_ROOM ||--o{ MEMBER_RETRO_ROOM : has_members
```

| Role | 권한 |
|------|------|
| `OWNER` | 수정, 삭제, AI 분석 |
| `MEMBER` | 참여만 가능 |

---

### 📝 Retrospect (회고)

```mermaid
erDiagram
    RETROSPECT {
        bigint retrospect_id PK
        bigint retro_room_id FK
        string title "회고 제목"
        string retrospect_method "KPT|4L|5F|PMI|FREE"
        text team_insight "AI 분석 결과"
        datetime created_at
        datetime updated_at
    }

    RETRO_ROOM ||--o{ RETROSPECT : contains
```

### 회고 방식 (Method)

```mermaid
flowchart LR
    subgraph methods["회고 방식"]
        direction TB
        KPT["KPT"]
        FOUR_L["FOUR_L (4L)"]
        FIVE_F["FIVE_F (5F)"]
        PMI["PMI"]
        FREE["FREE"]
    end

    KPT --- K["Keep: 유지할 것"]
    KPT --- P["Problem: 문제점"]
    KPT --- T["Try: 시도할 것"]

    FOUR_L --- L1["Liked: 좋았던 것"]
    FOUR_L --- L2["Learned: 배운 것"]
    FOUR_L --- L3["Lacked: 부족했던 것"]
    FOUR_L --- L4["Longed: 바라는 것"]
```

---

### 👤 Member Retro (회고 참여)

```mermaid
erDiagram
    MEMBER_RETRO {
        bigint id PK
        bigint member_id FK
        bigint retrospect_id FK
        string status "DRAFT|SUBMITTED|ANALYZED"
        text personal_insight "개인 인사이트"
        datetime submitted_at
        datetime created_at
    }

    MEMBER ||--o{ MEMBER_RETRO : participates
    RETROSPECT ||--o{ MEMBER_RETRO : has_participants
```

### 참여 상태 흐름

```mermaid
stateDiagram-v2
    [*] --> DRAFT: 참석 등록
    DRAFT --> SUBMITTED: 최종 제출
    SUBMITTED --> ANALYZED: AI 분석 완료
    ANALYZED --> [*]
```

---

### 💬 Response (회고 답변)

```mermaid
erDiagram
    RESPONSE {
        bigint response_id PK
        bigint retrospect_id FK
        bigint member_id FK "작성자"
        string question "질문"
        text content "답변 내용"
        datetime created_at
        datetime updated_at
    }

    RETROSPECT ||--o{ RESPONSE : has
```

---

### 💭 Response Comment (댓글)

```mermaid
erDiagram
    RESPONSE_COMMENT {
        bigint id PK
        bigint response_id FK
        bigint member_id FK
        text content "댓글 내용"
        datetime created_at
    }

    RESPONSE ||--o{ RESPONSE_COMMENT : has
    MEMBER ||--o{ RESPONSE_COMMENT : writes
```

---

### ❤️ Response Like (좋아요)

```mermaid
erDiagram
    RESPONSE_LIKE {
        bigint id PK
        bigint response_id FK
        bigint member_id FK
        datetime created_at
    }

    RESPONSE ||--o{ RESPONSE_LIKE : has
    MEMBER ||--o{ RESPONSE_LIKE : likes
```

> [!important] Unique Constraint
> `(response_id, member_id)` 조합은 유니크해야 함

---

### 📎 Retro Reference (참고자료)

```mermaid
erDiagram
    RETRO_REFERENCE {
        bigint id PK
        bigint retrospect_id FK
        string url "참고자료 URL"
        datetime created_at
    }

    RETROSPECT ||--o{ RETRO_REFERENCE : has
```

---

## 🗂️ 전체 ERD

```mermaid
erDiagram
    MEMBER ||--o{ REFRESH_TOKEN : has
    MEMBER ||--o{ MEMBER_RETRO_ROOM : joins
    MEMBER ||--o{ MEMBER_RETRO : participates
    MEMBER ||--o{ RESPONSE : writes
    MEMBER ||--o{ RESPONSE_COMMENT : comments
    MEMBER ||--o{ RESPONSE_LIKE : likes

    RETRO_ROOM ||--o{ RETROSPECT : contains
    RETRO_ROOM ||--o{ MEMBER_RETRO_ROOM : has_members

    RETROSPECT ||--o{ RESPONSE : has_answers
    RETROSPECT ||--o{ RETRO_REFERENCE : has_refs
    RETROSPECT ||--o{ MEMBER_RETRO : has_participants

    RESPONSE ||--o{ RESPONSE_COMMENT : has_comments
    RESPONSE ||--o{ RESPONSE_LIKE : has_likes

    MEMBER {
        bigint member_id PK
        string email UK
        string nickname
        string social_type
        int insight_count
    }

    REFRESH_TOKEN {
        bigint id PK
        bigint member_id FK
        string token UK
        datetime expires_at
    }

    RETRO_ROOM {
        bigint retro_room_id PK
        string title UK
        string description
        string invitation_url UK
    }

    MEMBER_RETRO_ROOM {
        bigint id PK
        bigint member_id FK
        bigint retro_room_id FK
        string role
        int order_index
    }

    RETROSPECT {
        bigint retrospect_id PK
        bigint retro_room_id FK
        string title
        string method
        text team_insight
    }

    MEMBER_RETRO {
        bigint id PK
        bigint member_id FK
        bigint retrospect_id FK
        string status
        text personal_insight
    }

    RESPONSE {
        bigint response_id PK
        bigint retrospect_id FK
        bigint member_id FK
        string question
        text content
    }

    RESPONSE_COMMENT {
        bigint id PK
        bigint response_id FK
        bigint member_id FK
        text content
    }

    RESPONSE_LIKE {
        bigint id PK
        bigint response_id FK
        bigint member_id FK
    }

    RETRO_REFERENCE {
        bigint id PK
        bigint retrospect_id FK
        string url
    }
```

---

## 🔄 Cascade Rules

| Parent | Child | On Delete |
|--------|-------|-----------|
| Member | RefreshToken | CASCADE |
| Member | MemberRetroRoom | SET NULL |
| Member | MemberRetro | SET NULL |
| RetroRoom | Retrospect | CASCADE |
| Retrospect | Response | CASCADE |
| Response | ResponseComment | CASCADE |
| Response | ResponseLike | CASCADE |

---

## 🔗 Related

- [[00-HOME|🏠 HOME]]
- [[01-Architecture|🏗️ Architecture]]
- [[05-API-Overview|🔌 API Overview]]

---

#entity #database #erd #schema
