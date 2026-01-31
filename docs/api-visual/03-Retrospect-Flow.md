# 📝 Retrospect Flow

> 회고방 생성부터 AI 분석까지 전체 회고 플로우

---

## 📍 Overview

```mermaid
flowchart TB
    subgraph phase1["1️⃣ 팀 설정"]
        CREATE_ROOM["회고방 생성"]
        INVITE["초대 코드 공유"]
        JOIN["팀원 합류"]
    end

    subgraph phase2["2️⃣ 회고 진행"]
        CREATE_RETRO["회고 생성"]
        REGISTER["참석 등록"]
        DRAFT["답변 작성"]
        SUBMIT["답변 제출"]
    end

    subgraph phase3["3️⃣ 분석 & 공유"]
        ANALYZE["AI 분석"]
        COMMENT["댓글/좋아요"]
        EXPORT["PDF 내보내기"]
    end

    phase1 --> phase2 --> phase3
```

---

## 1️⃣ 팀 설정 Phase

### 회고방 생성 → 팀원 합류

```mermaid
sequenceDiagram
    autonumber
    participant L as 👑 Leader
    participant S as 🦀 Server
    participant M as 👥 Members
    participant DB as 💾 Database

    Note over L,DB: 회고방 생성
    L->>S: POST /api/v1/retro-rooms
    Note right of L: { name, description }

    S->>S: Generate Invite Code
    Note over S: INV-XXXX-XXXX
    S->>DB: INSERT retro_room
    S->>DB: INSERT member_retro_room (OWNER)
    S-->>L: 201 Created
    Note left of S: { id, invitationUrl }

    Note over L,M: 초대 코드 공유
    L-->>M: Share: INV-XXXX-XXXX

    Note over M,DB: 팀원 합류
    M->>S: POST /api/v1/retro-rooms/join
    Note right of M: { invitationCode }

    S->>DB: SELECT retro_room WHERE invitation
    S->>DB: INSERT member_retro_room (MEMBER)
    S-->>M: 200 OK
    Note left of S: { retroRoom info }
```

### 초대 코드 형식

```
INV-XXXX-XXXX
     │    │
     │    └── 4자리 랜덤 (A-Z, 0-9)
     └── 4자리 랜덤 (A-Z, 0-9)
```

---

## 2️⃣ 회고 진행 Phase

### 회고 생성

```mermaid
sequenceDiagram
    autonumber
    participant L as 👑 Leader
    participant S as 🦀 Server
    participant DB as 💾 Database

    L->>S: POST /api/v1/retrospects
    Note right of L: { retroRoomId, title, method, references[] }

    S->>DB: Verify room ownership
    S->>DB: INSERT retrospect
    S->>DB: INSERT responses (기본 질문들)
    S->>DB: INSERT retro_references

    S-->>L: 201 Created
    Note left of S: { retrospectId }
```

### 회고 방식 (Method)

```mermaid
flowchart LR
    subgraph methods["회고 방식"]
        KPT["KPT"]
        FOUR_L["4L"]
        FIVE_F["5F"]
        PMI["PMI"]
        FREE["FREE"]
    end

    KPT --> Q1["Keep<br/>Problem<br/>Try"]
    FOUR_L --> Q2["Liked<br/>Learned<br/>Lacked<br/>Longed"]
    FIVE_F --> Q3["Facts<br/>Feelings<br/>Findings<br/>Future<br/>Feedback"]
    PMI --> Q4["Plus<br/>Minus<br/>Interesting"]
    FREE --> Q5["자유 질문<br/>5개"]
```

---

### 답변 작성 → 제출 플로우

```mermaid
sequenceDiagram
    autonumber
    participant M as 👤 Member
    participant S as 🦀 Server
    participant DB as 💾 Database

    Note over M,DB: 참석 등록
    M->>S: POST /api/v1/retrospects/{id}/participants
    S->>DB: INSERT member_retro (status: DRAFT)
    S-->>M: 200 OK

    Note over M,DB: 임시 저장 (반복)
    loop 작성 중
        M->>S: PUT /api/v1/retrospects/{id}/drafts
        Note right of M: { responses: [{ questionId, content }] }
        S->>DB: UPDATE responses
        S-->>M: 200 OK
    end

    Note over M,DB: 최종 제출
    M->>S: POST /api/v1/retrospects/{id}/submit

    S->>DB: Validate all responses filled

    alt ❌ 미작성 질문 있음
        S-->>M: 400 Bad Request
        Note left of S: RETRO4001
    end

    S->>DB: UPDATE member_retro SET status = SUBMITTED
    S-->>M: 200 OK
```

### 회고 상태 흐름

```mermaid
stateDiagram-v2
    [*] --> DRAFT: 참석 등록
    DRAFT --> DRAFT: 임시 저장
    DRAFT --> SUBMITTED: 최종 제출
    SUBMITTED --> ANALYZED: AI 분석
    ANALYZED --> [*]

    note right of DRAFT: 수정 가능
    note right of SUBMITTED: 수정 불가
    note right of ANALYZED: 인사이트 확인 가능
```

---

## 3️⃣ 분석 & 공유 Phase

### AI 분석 플로우

```mermaid
sequenceDiagram
    autonumber
    participant L as 👑 Leader
    participant S as 🦀 Server
    participant AI as 🤖 OpenAI
    participant DB as 💾 Database

    L->>S: POST /api/v1/retrospects/{id}/analysis

    S->>DB: Check owner permission
    S->>DB: Check monthly limit

    alt ❌ 월간 한도 초과
        S-->>L: 403 Forbidden
        Note left of S: AI4031
    end

    S->>DB: Get all SUBMITTED responses

    alt ❌ 제출된 응답 없음
        S-->>L: 400 Bad Request
        Note left of S: AI4002
    end

    S->>AI: Analyze Request
    Note over S,AI: System Prompt + User Data
    AI-->>S: Analysis Result

    S->>DB: UPDATE retrospect SET team_insight
    S->>DB: UPDATE member_retro SET personal_insight
    S->>DB: UPDATE member_retro SET status = ANALYZED

    S-->>L: 200 OK
    Note left of S: { teamInsight, emotionRank, missions }
```

### AI 분석 결과 구조

```mermaid
flowchart TB
    subgraph result["AI 분석 결과"]
        TEAM["팀 인사이트<br/>team_insight"]
        EMOTION["감정 순위<br/>emotion_rank"]
        T_MISSION["팀 미션<br/>team_missions"]
        P_MISSION["개인 미션<br/>personal_missions"]
    end

    TEAM --> SUMMARY["팀 전체<br/>종합 분석"]
    EMOTION --> TOP3["상위 3개<br/>감정 + 이유"]
    T_MISSION --> TASKS["3개 팀<br/>액션 아이템"]
    P_MISSION --> PERSONAL["멤버별<br/>3개 미션"]
```

---

## 📊 전체 상태 다이어그램

```mermaid
stateDiagram-v2
    direction LR

    state "회고방" as ROOM {
        [*] --> Created: 생성
        Created --> Active: 팀원 합류
        Active --> [*]
    }

    state "회고" as RETRO {
        [*] --> Preparing: 생성
        Preparing --> InProgress: 참석 등록
        InProgress --> Reviewing: 전원 제출
        Reviewing --> Completed: AI 분석
        Completed --> [*]
    }

    state "개인 답변" as ANSWER {
        [*] --> Draft: 참석 등록
        Draft --> Draft: 임시 저장
        Draft --> Submitted: 최종 제출
        Submitted --> Analyzed: 분석 완료
        Analyzed --> [*]
    }

    ROOM --> RETRO
    RETRO --> ANSWER
```

---

## 🔄 연관 API 맵

```mermaid
flowchart TB
    subgraph team["👥 Team APIs"]
        A5["API-005<br/>회고방 생성"]
        A6["API-006<br/>팀 합류"]
        A7["API-007<br/>팀 목록"]
        A8["API-008<br/>순서 변경"]
        A9["API-009<br/>이름 변경"]
        A10["API-010<br/>팀 삭제"]
        A11["API-011<br/>회고 목록"]
    end

    subgraph retro["📝 Retrospect APIs"]
        A12["API-012<br/>회고 생성"]
        A13["API-013<br/>회고 상세"]
        A14["API-014<br/>회고 삭제"]
        A15["API-015<br/>참석 등록"]
        A16["API-016<br/>임시 저장"]
        A17["API-017<br/>최종 제출"]
    end

    subgraph content["📄 Content APIs"]
        A18["API-018<br/>참고자료"]
        A19["API-019<br/>보관함"]
        A20["API-020<br/>카테고리별"]
        A21["API-021<br/>PDF 내보내기"]
        A22["API-022<br/>AI 분석"]
        A23["API-023<br/>검색"]
    end

    subgraph social["💬 Social APIs"]
        A25["API-025<br/>좋아요"]
        A26["API-026<br/>댓글 조회"]
        A27["API-027<br/>댓글 작성"]
    end

    A5 --> A6 --> A11 --> A12
    A12 --> A15 --> A16 --> A17
    A17 --> A22
    A22 --> A20
    A20 --> A25 & A26
    A26 --> A27
    A22 --> A21
```

---

## 🔗 Related

- [[00-HOME|🏠 HOME]]
- [[02-Auth-Flow|🔐 Auth Flow]] ←
- [[06-AI-Analysis-Flow|🤖 AI Analysis Flow]] →
- [[08-Team-APIs|👥 Team APIs]]
- [[09-Retrospect-APIs|📝 Retrospect APIs]]

---

#retrospect #flow #team #ai #analysis
