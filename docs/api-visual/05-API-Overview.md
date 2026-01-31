# 🔌 API Overview

> 전체 API 엔드포인트 맵 및 카테고리별 정리

---

## 📍 API Map

```mermaid
flowchart TB
    subgraph BASE["/api/v1"]
        direction TB

        subgraph AUTH["/auth"]
            A1["POST /social-login"]
            A2["POST /signup"]
            A3["POST /token/refresh"]
            A4["POST /logout"]
        end

        subgraph ROOMS["/retro-rooms"]
            R1["POST /"]
            R2["GET /"]
            R3["POST /join"]
            R4["PATCH /order"]
            R5["PATCH /:id/name"]
            R6["DELETE /:id"]
            R7["GET /:id/retrospects"]
        end

        subgraph RETROS["/retrospects"]
            RT1["POST /"]
            RT2["GET /:id"]
            RT3["DELETE /:id"]
            RT4["POST /:id/participants"]
            RT5["PUT /:id/drafts"]
            RT6["POST /:id/submit"]
            RT7["GET /:id/references"]
            RT8["GET /storage"]
            RT9["GET /:id/responses"]
            RT10["GET /:id/export"]
            RT11["POST /:id/analysis"]
            RT12["GET /search"]
        end

        subgraph RESPONSES["/responses"]
            RS1["POST /:id/likes"]
            RS2["GET /:id/comments"]
            RS3["POST /:id/comments"]
        end

        subgraph MEMBERS["/members"]
            M1["DELETE /me"]
        end
    end
```

---

## 📋 API 전체 목록

### 🔐 Auth (인증)

| ID | Method | Endpoint | 설명 | Auth |
|:--:|:------:|----------|------|:----:|
| 001 | 🟡 POST | `/api/v1/auth/social-login` | 소셜 로그인 | 🔓 |
| 002 | 🟡 POST | `/api/v1/auth/signup` | 회원가입 | 🔓 |
| 003 | 🟡 POST | `/api/v1/auth/token/refresh` | 토큰 갱신 | 🔓 |
| 004 | 🟡 POST | `/api/v1/auth/logout` | 로그아웃 | 🔐 |

```mermaid
flowchart LR
    A1["소셜 로그인"]
    A2["회원가입"]
    A3["토큰 갱신"]
    A4["로그아웃"]

    A1 -->|신규회원| A2
    A1 -->|기존회원| AT["Access Token"]
    A2 --> AT
    AT -->|만료| A3
    AT --> A4
```

→ [[07-Auth-APIs|상세 보기]]

---

### 👥 Team (회고방)

| ID | Method | Endpoint | 설명 | Auth |
|:--:|:------:|----------|------|:----:|
| 005 | 🟡 POST | `/api/v1/retro-rooms` | 회고방 생성 | 🔐 |
| 006 | 🟡 POST | `/api/v1/retro-rooms/join` | 회고방 참여 | 🔐 |
| 007 | 🟢 GET | `/api/v1/retro-rooms` | 회고방 목록 | 🔐 |
| 008 | 🟣 PATCH | `/api/v1/retro-rooms/order` | 순서 변경 | 🔐 |
| 009 | 🟣 PATCH | `/api/v1/retro-rooms/:id/name` | 이름 변경 | 👑 |
| 010 | 🔴 DELETE | `/api/v1/retro-rooms/:id` | 회고방 삭제 | 👑 |
| 011 | 🟢 GET | `/api/v1/retro-rooms/:id/retrospects` | 회고 목록 | 🔐 |

```mermaid
flowchart TB
    CREATE["생성"] --> LIST["목록 조회"]
    CREATE --> INVITE["초대 코드"]
    INVITE --> JOIN["팀원 합류"]
    JOIN --> LIST
    LIST --> RETROS["회고 목록"]
    LIST --> ORDER["순서 변경"]
    LIST --> RENAME["이름 변경"]
    LIST --> DELETE["삭제"]
```

→ [[08-Team-APIs|상세 보기]]

---

### 📝 Retrospect (회고)

| ID | Method | Endpoint | 설명 | Auth |
|:--:|:------:|----------|------|:----:|
| 012 | 🟡 POST | `/api/v1/retrospects` | 회고 생성 | 👑 |
| 013 | 🟢 GET | `/api/v1/retrospects/:id` | 회고 상세 | 🔐 |
| 014 | 🔴 DELETE | `/api/v1/retrospects/:id` | 회고 삭제 | 👑 |
| 015 | 🟡 POST | `/api/v1/retrospects/:id/participants` | 참석 등록 | 🔐 |
| 016 | 🔵 PUT | `/api/v1/retrospects/:id/drafts` | 임시 저장 | 🔐 |
| 017 | 🟡 POST | `/api/v1/retrospects/:id/submit` | 최종 제출 | 🔐 |
| 018 | 🟢 GET | `/api/v1/retrospects/:id/references` | 참고자료 | 🔐 |
| 019 | 🟢 GET | `/api/v1/retrospects/storage` | 보관함 | 🔐 |
| 020 | 🟢 GET | `/api/v1/retrospects/:id/responses` | 카테고리별 답변 | 🔐 |
| 021 | 🟢 GET | `/api/v1/retrospects/:id/export` | PDF 내보내기 | 🔐 |
| 022 | 🟡 POST | `/api/v1/retrospects/:id/analysis` | AI 분석 | 👑 |
| 023 | 🟢 GET | `/api/v1/retrospects/search` | 검색 | 🔐 |

```mermaid
flowchart TB
    subgraph create["생성"]
        C1["회고 생성"]
        C2["참석 등록"]
    end

    subgraph write["작성"]
        W1["임시 저장"]
        W2["최종 제출"]
    end

    subgraph view["조회"]
        V1["상세 조회"]
        V2["참고자료"]
        V3["보관함"]
        V4["카테고리별"]
        V5["검색"]
    end

    subgraph analyze["분석"]
        A1["AI 분석"]
        A2["PDF 내보내기"]
    end

    C1 --> C2 --> W1 --> W2 --> A1
    W2 --> V4
    A1 --> V1 & A2
```

→ [[09-Retrospect-APIs|상세 보기]]

---

### 💬 Social (소셜)

| ID | Method | Endpoint | 설명 | Auth |
|:--:|:------:|----------|------|:----:|
| 026 | 🟡 POST | `/api/v1/responses/:id/likes` | 좋아요 토글 | 🔐 |
| 027 | 🟢 GET | `/api/v1/responses/:id/comments` | 댓글 조회 | 🔐 |
| 028 | 🟡 POST | `/api/v1/responses/:id/comments` | 댓글 작성 | 🔐 |

```mermaid
flowchart LR
    RESPONSE["답변"]
    LIKE["좋아요"]
    COMMENT["댓글"]

    RESPONSE --> LIKE
    RESPONSE --> COMMENT
```

→ [[10-Social-APIs|상세 보기]]

---

### 👤 Member (회원)

| ID | Method | Endpoint | 설명 | Auth |
|:--:|:------:|----------|------|:----:|
| 025 | 🔴 DELETE | `/api/v1/members/me` | 서비스 탈퇴 | 🔐 |

---

## 🎯 권한 매트릭스

```mermaid
flowchart TB
    subgraph public["🔓 Public (인증 불필요)"]
        P1["소셜 로그인"]
        P2["회원가입"]
        P3["토큰 갱신"]
    end

    subgraph member["🔐 Member (로그인 필요)"]
        M1["회고방 생성/참여"]
        M2["회고 참여/작성"]
        M3["댓글/좋아요"]
        M4["조회 기능"]
    end

    subgraph owner["👑 Owner (방장 권한)"]
        O1["회고방 수정/삭제"]
        O2["회고 생성/삭제"]
        O3["AI 분석"]
    end

    public --> member --> owner
```

| 권한 | 설명 | API 예시 |
|------|------|---------|
| 🔓 Public | 인증 불필요 | 로그인, 회원가입 |
| 🔐 Member | Access Token 필요 | 대부분의 API |
| 👑 Owner | 회고방 소유자만 | 수정, 삭제, AI 분석 |

---

## 📊 HTTP Method 분포

```mermaid
pie title HTTP Methods
    "GET" : 12
    "POST" : 12
    "PUT" : 1
    "PATCH" : 2
    "DELETE" : 3
```

| Method | 개수 | 용도 |
|--------|------|------|
| 🟢 GET | 12 | 조회 |
| 🟡 POST | 12 | 생성/액션 |
| 🔵 PUT | 1 | 전체 수정 |
| 🟣 PATCH | 2 | 부분 수정 |
| 🔴 DELETE | 3 | 삭제 |

---

## 🚨 공통 에러 코드

| Code | HTTP | 설명 |
|------|------|------|
| COMMON200 | 200 | 성공 |
| COMMON400 | 400 | 잘못된 요청 |
| COMMON401 | 401 | 인증 실패 |
| COMMON403 | 403 | 권한 없음 |
| COMMON404 | 404 | 리소스 없음 |
| COMMON500 | 500 | 서버 에러 |

→ [[11-Error-Codes|에러 코드 전체 목록]]

---

## 🔗 Related

- [[00-HOME|🏠 HOME]]
- [[01-Architecture|🏗️ Architecture]]
- [[apis/MOC|📑 API 목록 (MOC)]]

---

#api #endpoint #overview #map
