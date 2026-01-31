# 📝 Retrospect APIs

> 회고 관련 API 상세 명세

---

## 📍 Overview

```mermaid
flowchart TB
    subgraph lifecycle["회고 라이프사이클"]
        direction LR
        CREATE["생성<br/>API-012"]
        REGISTER["참석 등록<br/>API-015"]
        DRAFT["임시 저장<br/>API-016"]
        SUBMIT["제출<br/>API-017"]
        ANALYZE["AI 분석<br/>API-022"]
    end

    CREATE --> REGISTER --> DRAFT --> SUBMIT --> ANALYZE

    subgraph view["조회"]
        V1["상세<br/>API-013"]
        V2["참고자료<br/>API-018"]
        V3["보관함<br/>API-019"]
        V4["카테고리별<br/>API-020"]
        V5["검색<br/>API-023"]
    end

    subgraph export["내보내기"]
        E1["PDF<br/>API-021"]
    end

    subgraph manage["관리"]
        M1["삭제<br/>API-014"]
    end

    ANALYZE --> view
    ANALYZE --> export
```

---

## 🔄 회고 상태 흐름

```mermaid
stateDiagram-v2
    [*] --> CREATED: 회고 생성

    state 참여자상태 {
        [*] --> DRAFT: 참석 등록
        DRAFT --> DRAFT: 임시 저장
        DRAFT --> SUBMITTED: 최종 제출
        SUBMITTED --> ANALYZED: AI 분석
    }

    CREATED --> 참여자상태
    ANALYZED --> [*]
```

---

## API-012 회고 생성

> `POST /api/v1/retrospects` 👑

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: { retroRoomId, title, method, references }
    S->>DB: Check OWNER role

    alt 권한 없음
        S-->>C: 403 Forbidden
    end

    S->>DB: INSERT retrospect
    S->>DB: INSERT default responses
    S->>DB: INSERT references
    S-->>C: 201 Created
```

### Request

```json
{
  "retroRoomId": 1,
  "title": "스프린트 1 회고",
  "retrospectMethod": "KPT",
  "references": [
    { "url": "https://notion.so/sprint1" }
  ]
}
```

### 회고 방식

| Method | 기본 질문 |
|--------|---------|
| KPT | Keep, Problem, Try |
| FOUR_L | Liked, Learned, Lacked, Longed |
| FIVE_F | Facts, Feelings, Findings, Future, Feedback |
| PMI | Plus, Minus, Interesting |
| FREE | 자유 질문 5개 |

→ [[apis/API-012 회고 생성|상세 문서]]

---

## API-013 회고 상세

> `GET /api/v1/retrospects/:id` 🔐

### Response

```json
{
  "retrospectId": 1,
  "title": "스프린트 1 회고",
  "method": "KPT",
  "teamInsight": "팀 전체 인사이트...",
  "questions": [
    {
      "questionId": 1,
      "question": "Keep: 유지하고 싶은 점은?",
      "myAnswer": "커뮤니케이션..."
    }
  ],
  "participants": [
    {
      "memberId": 1,
      "nickname": "홍길동",
      "status": "ANALYZED"
    }
  ]
}
```

→ [[apis/API-013 회고 상세|상세 문서]]

---

## API-014 회고 삭제

> `DELETE /api/v1/retrospects/:id` 👑

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: DELETE /retrospects/1
    S->>DB: Check OWNER role
    S->>DB: DELETE retrospect (CASCADE)
    S-->>C: 200 OK
```

→ [[apis/API-014 회고 삭제|상세 문서]]

---

## API-015 참석 등록

> `POST /api/v1/retrospects/:id/participants` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: POST /retrospects/1/participants
    S->>DB: Check membership

    alt 이미 참석
        S-->>C: 409 Conflict
    end

    S->>DB: INSERT member_retro (DRAFT)
    S-->>C: 200 OK
```

→ [[apis/API-015 참석 등록|상세 문서]]

---

## API-016 임시 저장

> `PUT /api/v1/retrospects/:id/drafts` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: { responses: [...] }
    S->>DB: Check status == DRAFT

    alt 이미 제출됨
        S-->>C: 403 Forbidden
    end

    loop 각 응답
        S->>DB: UPDATE response
    end

    S-->>C: 200 OK
```

### Request

```json
{
  "responses": [
    {
      "questionId": 1,
      "content": "Keep: 팀 커뮤니케이션이 좋았습니다..."
    },
    {
      "questionId": 2,
      "content": "Problem: 일정 관리가 어려웠습니다..."
    }
  ]
}
```

→ [[apis/API-017 임시 저장|상세 문서]]

---

## API-017 최종 제출

> `POST /api/v1/retrospects/:id/submit` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: POST /retrospects/1/submit
    S->>DB: Check all responses filled

    alt 미작성 질문 있음
        S-->>C: 400 Bad Request
    end

    S->>DB: UPDATE status = SUBMITTED
    S->>DB: SET submitted_at
    S-->>C: 200 OK
```

### Validation

| 항목 | 조건 |
|------|------|
| 모든 질문 | 답변 필수 |
| 답변 길이 | 최소 1자 이상 |

→ [[apis/API-018 최종 제출|상세 문서]]

---

## API-018 참고자료 조회

> `GET /api/v1/retrospects/:id/references` 🔐

### Response

```json
{
  "references": [
    {
      "id": 1,
      "url": "https://notion.so/sprint1",
      "createdAt": "2024-01-15T10:00:00Z"
    }
  ]
}
```

→ [[apis/API-019 참고자료|상세 문서]]

---

## API-019 보관함

> `GET /api/v1/retrospects/storage` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: GET /retrospects/storage
    S->>DB: SELECT retrospects<br/>WHERE member participated<br/>AND status = ANALYZED
    S-->>C: { retrospects: [...] }
```

### Response

```json
{
  "retrospects": [
    {
      "retrospectId": 1,
      "title": "스프린트 1 회고",
      "retroRoomName": "우리 팀",
      "analyzedAt": "2024-01-20T10:00:00Z"
    }
  ]
}
```

→ [[apis/API-020 보관함|상세 문서]]

---

## API-020 카테고리별 답변 조회

> `GET /api/v1/retrospects/:id/responses` 🔐

### Query Parameters

| Param | Type | 설명 |
|-------|------|------|
| `category` | String | 질문 카테고리 (선택) |

### Response

```json
{
  "responses": [
    {
      "questionId": 1,
      "question": "Keep: 유지하고 싶은 점은?",
      "answers": [
        {
          "memberId": 1,
          "nickname": "홍길동",
          "content": "커뮤니케이션...",
          "likeCount": 3,
          "isLiked": true
        }
      ]
    }
  ]
}
```

→ [[apis/API-021 카테고리별 조회|상세 문서]]

---

## API-021 PDF 내보내기

> `GET /api/v1/retrospects/:id/export` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server

    C->>S: GET /retrospects/1/export
    S->>S: Generate PDF
    Note over S: retrospect_20240115_120000.pdf
    S-->>C: PDF File (binary)
```

### Response Headers

```
Content-Type: application/pdf
Content-Disposition: attachment; filename="retrospect_20240115_120000.pdf"
```

→ [[apis/API-022 PDF 내보내기|상세 문서]]

---

## API-022 AI 분석

> `POST /api/v1/retrospects/:id/analysis` 👑

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant AI as OpenAI
    participant DB as Database

    C->>S: POST /retrospects/1/analysis
    S->>DB: Check conditions
    S->>AI: Analyze request
    AI-->>S: Analysis result
    S->>DB: Save insights
    S-->>C: { teamInsight, emotionRank, missions }
```

### Conditions

| 조건 | 에러 |
|------|------|
| Owner 권한 | RETRO4031 |
| 제출된 답변 있음 | AI4002 |
| 월간 한도 내 | AI4031 |
| 미분석 상태 | RETRO4091 |

→ [[apis/API-023 AI 분석|상세 문서]]
→ [[06-AI-Analysis-Flow|AI 분석 상세 플로우]]

---

## API-023 검색

> `GET /api/v1/retrospects/search` 🔐

### Query Parameters

| Param | Type | 설명 |
|-------|------|------|
| `keyword` | String | 검색어 |
| `method` | String | 회고 방식 필터 |
| `startDate` | Date | 시작일 |
| `endDate` | Date | 종료일 |

### Response

```json
{
  "retrospects": [
    {
      "retrospectId": 1,
      "title": "스프린트 1 회고",
      "retroRoomName": "우리 팀",
      "method": "KPT",
      "createdAt": "2024-01-15T10:00:00Z"
    }
  ]
}
```

→ [[apis/API-024 검색|상세 문서]]

---

## 🚨 Error Codes

| Code | HTTP | 설명 |
|------|------|------|
| RETRO4001 | 400 | 미작성 질문 존재 |
| RETRO4031 | 403 | 권한 없음 |
| RETRO4033 | 403 | 이미 제출됨 |
| RETRO4041 | 404 | 회고 없음 |
| RETRO4091 | 409 | 이미 분석됨 |

---

## 🔗 Related

- [[00-HOME|🏠 HOME]]
- [[03-Retrospect-Flow|📝 Retrospect Flow]]
- [[06-AI-Analysis-Flow|🤖 AI Analysis Flow]]
- [[05-API-Overview|🔌 API Overview]]

---

#retrospect #api #crud #ai
