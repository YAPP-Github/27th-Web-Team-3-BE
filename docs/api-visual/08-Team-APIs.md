# 👥 Team APIs

> 회고방(팀) 관련 API 상세 명세

---

## 📍 Overview

```mermaid
flowchart TB
    subgraph create["생성"]
        A5["API-005<br/>회고방 생성"]
    end

    subgraph join["참여"]
        A6["API-006<br/>팀 합류"]
    end

    subgraph manage["관리"]
        A7["API-007<br/>목록 조회"]
        A8["API-008<br/>순서 변경"]
        A9["API-009<br/>이름 변경"]
        A10["API-010<br/>삭제"]
    end

    subgraph content["컨텐츠"]
        A11["API-011<br/>회고 목록"]
    end

    A5 --> A7
    A6 --> A7
    A7 --> A8 & A9 & A10 & A11
```

---

## 권한 매트릭스

| API | Member | Owner |
|-----|:------:|:-----:|
| 회고방 생성 | ✅ | - |
| 팀 합류 | ✅ | - |
| 목록 조회 | ✅ | ✅ |
| 순서 변경 | ✅ | ✅ |
| 이름 변경 | ❌ | ✅ |
| 회고방 삭제 | ❌ | ✅ |
| 회고 목록 | ✅ | ✅ |

---

## API-005 회고방 생성

> `POST /api/v1/retro-rooms` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: { name, description }
    S->>S: Generate Invite Code
    Note over S: INV-XXXX-XXXX
    S->>DB: INSERT retro_room
    S->>DB: INSERT member_retro_room (OWNER)
    S-->>C: 201 Created
```

### Request

```json
{
  "name": "우리 팀 회고방",
  "description": "스프린트 회고를 위한 공간입니다"
}
```

### Response

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "retroRoomId": 1,
    "name": "우리 팀 회고방",
    "invitationUrl": "INV-A1B2-C3D4"
  }
}
```

→ [[apis/API-005 팀 생성|상세 문서]]

---

## API-006 팀 합류

> `POST /api/v1/retro-rooms/join` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: { invitationCode }
    S->>DB: SELECT retro_room

    alt 코드 없음
        S-->>C: 404 Not Found
    end

    S->>DB: Check already joined

    alt 이미 참여중
        S-->>C: 409 Conflict
    end

    S->>DB: INSERT member_retro_room (MEMBER)
    S-->>C: 200 OK
```

### Request

```json
{
  "invitationCode": "INV-A1B2-C3D4"
}
```

### Response

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "retroRoomId": 1,
    "name": "우리 팀 회고방"
  }
}
```

### Errors

| Code | HTTP | 설명 |
|------|------|------|
| RETRO4041 | 404 | 존재하지 않는 초대 코드 |
| RETRO4091 | 409 | 이미 참여한 회고방 |

→ [[apis/API-006 팀 합류|상세 문서]]

---

## API-007 팀 목록

> `GET /api/v1/retro-rooms` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: GET (with accessToken)
    S->>DB: SELECT retro_rooms<br/>JOIN member_retro_room
    DB-->>S: rooms with role
    S-->>C: { retroRooms: [...] }
```

### Response

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "retroRooms": [
      {
        "retroRoomId": 1,
        "name": "우리 팀 회고방",
        "role": "OWNER",
        "orderIndex": 0,
        "memberCount": 5
      }
    ]
  }
}
```

→ [[apis/API-007 팀 목록|상세 문서]]

---

## API-008 팀 순서 변경

> `PATCH /api/v1/retro-rooms/order` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: { orders: [...] }

    loop 각 회고방
        S->>DB: UPDATE order_index
    end

    S-->>C: 200 OK
```

### Request

```json
{
  "orders": [
    { "retroRoomId": 3, "orderIndex": 0 },
    { "retroRoomId": 1, "orderIndex": 1 },
    { "retroRoomId": 2, "orderIndex": 2 }
  ]
}
```

→ [[apis/API-008 팀 순서 변경|상세 문서]]

---

## API-009 팀 이름 변경

> `PATCH /api/v1/retro-rooms/:id/name` 👑

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: { name: "새 이름" }
    S->>DB: Check OWNER role

    alt 권한 없음
        S-->>C: 403 Forbidden
    end

    S->>DB: UPDATE retro_room SET name
    S-->>C: 200 OK
```

### Request

```json
{
  "name": "새로운 회고방 이름"
}
```

### Errors

| Code | HTTP | 설명 |
|------|------|------|
| RETRO4031 | 403 | 회고방 수정 권한 없음 |

→ [[apis/API-009 팀 이름 변경|상세 문서]]

---

## API-010 팀 삭제

> `DELETE /api/v1/retro-rooms/:id` 👑

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: DELETE /retro-rooms/1
    S->>DB: Check OWNER role

    alt 권한 없음
        S-->>C: 403 Forbidden
    end

    S->>DB: DELETE retro_room (CASCADE)
    Note over DB: 연관 데이터 삭제:<br/>retrospects, responses...
    S-->>C: 200 OK
```

### Errors

| Code | HTTP | 설명 |
|------|------|------|
| RETRO4031 | 403 | 회고방 삭제 권한 없음 |

→ [[apis/API-010 팀 삭제|상세 문서]]

---

## API-011 회고 목록

> `GET /api/v1/retro-rooms/:id/retrospects` 🔐

### 흐름

```mermaid
sequenceDiagram
    participant C as Client
    participant S as Server
    participant DB as Database

    C->>S: GET /retro-rooms/1/retrospects
    S->>DB: Check membership
    S->>DB: SELECT retrospects
    S-->>C: { retrospects: [...] }
```

### Response

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "result": {
    "retrospects": [
      {
        "retrospectId": 1,
        "title": "스프린트 1 회고",
        "method": "KPT",
        "status": "ANALYZED",
        "participantCount": 5,
        "createdAt": "2024-01-15T10:00:00Z"
      }
    ]
  }
}
```

→ [[apis/API-011 팀 회고 목록|상세 문서]]

---

## 📊 회고방 상태

```mermaid
stateDiagram-v2
    [*] --> Created: 생성
    Created --> Active: 멤버 참여
    Active --> Active: 회고 진행
    Active --> [*]: 삭제
```

---

## 🔗 Related

- [[00-HOME|🏠 HOME]]
- [[03-Retrospect-Flow|📝 Retrospect Flow]]
- [[05-API-Overview|🔌 API Overview]]

---

#team #retro-room #api
