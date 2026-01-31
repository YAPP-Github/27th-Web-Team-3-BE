# 🚨 Error Codes

> 전체 에러 코드 참조 문서

---

## 📍 Overview

```mermaid
flowchart TB
    subgraph codes["에러 코드 체계"]
        COMMON["COMMON<br/>공통"]
        AUTH["AUTH<br/>인증"]
        MEMBER["MEMBER<br/>회원"]
        RETRO["RETRO<br/>회고"]
        RES["RES<br/>답변"]
        AI["AI<br/>분석"]
        SEARCH["SEARCH<br/>검색"]
    end

    subgraph http["HTTP Status"]
        H2XX["2XX<br/>성공"]
        H4XX["4XX<br/>클라이언트 에러"]
        H5XX["5XX<br/>서버 에러"]
    end

    codes --> http
```

---

## 📋 공통 에러 (COMMON)

| Code | HTTP | 설명 | 대응 |
|------|------|------|------|
| COMMON200 | 200 | 성공 | - |
| COMMON400 | 400 | 잘못된 요청 | 요청 데이터 확인 |
| COMMON401 | 401 | 인증 실패 | 로그인 필요 |
| COMMON403 | 403 | 권한 없음 | 권한 확인 |
| COMMON404 | 404 | 리소스 없음 | URL 확인 |
| COMMON500 | 500 | 서버 에러 | 관리자 문의 |

```mermaid
flowchart LR
    REQ["요청"] --> CHECK{"유효?"}
    CHECK -->|Yes| PROC["처리"]
    CHECK -->|No| C400["COMMON400"]
    PROC --> AUTH{"인증?"}
    AUTH -->|No| C401["COMMON401"]
    AUTH -->|Yes| PERM{"권한?"}
    PERM -->|No| C403["COMMON403"]
    PERM -->|Yes| FIND{"존재?"}
    FIND -->|No| C404["COMMON404"]
    FIND -->|Yes| EXEC["실행"]
    EXEC -->|에러| C500["COMMON500"]
    EXEC -->|성공| C200["COMMON200"]
```

---

## 🔐 인증 에러 (AUTH)

| Code | HTTP | 설명 | 발생 상황 | 대응 |
|------|------|------|----------|------|
| AUTH2001 | 200 | 신규 회원 | 소셜 로그인 시 미가입 | 회원가입 진행 |
| AUTH4001 | 401 | 인증 실패 | 토큰 없음/만료 | 재로그인 |
| AUTH4002 | 401 | 무효한 소셜 토큰 | 소셜 토큰 검증 실패 | 소셜 재인증 |
| AUTH4003 | 400 | 무효한 회원가입 토큰 | 만료/조작된 signupToken | 로그인 재시도 |
| AUTH4004 | 401 | 무효한 리프레시 토큰 | 만료/조작된 refreshToken | 재로그인 |
| AUTH4005 | 401 | 로그아웃된 토큰 | DB에서 삭제된 토큰 | 재로그인 |
| AUTH4091 | 409 | 이미 가입된 이메일 | 중복 회원가입 시도 | 로그인 시도 |

```mermaid
flowchart TB
    LOGIN["로그인 시도"]
    LOGIN --> SOCIAL{"소셜 토큰<br/>유효?"}
    SOCIAL -->|No| AUTH4002["AUTH4002"]
    SOCIAL -->|Yes| MEMBER{"회원 존재?"}
    MEMBER -->|No| AUTH2001["AUTH2001<br/>(정상)"]
    MEMBER -->|Yes| TOKEN["토큰 발급"]

    SIGNUP["회원가입"]
    SIGNUP --> ST{"signupToken<br/>유효?"}
    ST -->|No| AUTH4003["AUTH4003"]
    ST -->|Yes| DUP{"중복?"}
    DUP -->|Yes| AUTH4091["AUTH4091"]
    DUP -->|No| SUCCESS["성공"]

    REFRESH["토큰 갱신"]
    REFRESH --> RT{"refreshToken<br/>유효?"}
    RT -->|No| AUTH4004["AUTH4004"]
    RT -->|Yes| DB{"DB에<br/>존재?"}
    DB -->|No| AUTH4005["AUTH4005"]
    DB -->|Yes| NEW["새 토큰"]
```

---

## 👤 회원 에러 (MEMBER)

| Code | HTTP | 설명 | 발생 상황 |
|------|------|------|----------|
| MEMBER4041 | 404 | 회원 없음 | 탈퇴/존재하지 않는 회원 |

---

## 📝 회고 에러 (RETRO)

| Code | HTTP | 설명 | 발생 상황 | 대응 |
|------|------|------|----------|------|
| RETRO4001 | 400 | 미작성 질문 존재 | 제출 시 빈 답변 | 모든 질문 작성 |
| RETRO4031 | 403 | 회고방 권한 없음 | Owner 아닌 사용자 | 방장에게 요청 |
| RETRO4033 | 403 | 이미 제출됨 | 제출 후 수정 시도 | 수정 불가 안내 |
| RETRO4041 | 404 | 회고 없음 | 삭제/없는 회고 | ID 확인 |
| RETRO4091 | 409 | 이미 분석됨 | 중복 분석 시도 | 결과 확인 |

```mermaid
flowchart TB
    subgraph submit["제출 시"]
        CHECK_FILLED{"모든 질문<br/>작성?"}
        CHECK_FILLED -->|No| R4001["RETRO4001"]
        CHECK_FILLED -->|Yes| CHECK_STATUS{"상태가<br/>DRAFT?"}
        CHECK_STATUS -->|No| R4033["RETRO4033"]
        CHECK_STATUS -->|Yes| SUBMIT_OK["제출 성공"]
    end

    subgraph analyze["분석 시"]
        CHECK_OWNER{"Owner?"}
        CHECK_OWNER -->|No| R4031["RETRO4031"]
        CHECK_OWNER -->|Yes| CHECK_ANALYZED{"이미<br/>분석?"}
        CHECK_ANALYZED -->|Yes| R4091["RETRO4091"]
        CHECK_ANALYZED -->|No| ANALYZE_OK["분석 진행"]
    end
```

---

## 💬 답변 에러 (RES)

| Code | HTTP | 설명 | 발생 상황 |
|------|------|------|----------|
| RES4041 | 404 | 답변 없음 | 삭제/없는 답변 |

---

## 🤖 AI 에러 (AI)

| Code | HTTP | 설명 | 발생 상황 | 대응 |
|------|------|------|----------|------|
| AI4002 | 400 | 분석 데이터 부족 | 제출된 답변 없음 | 팀원 제출 유도 |
| AI4031 | 403 | 월간 한도 초과 | 10회 초과 | 다음 달 대기 |
| AI5001 | 500 | AI 분석 실패 | OpenAI 응답 파싱 실패 | 재시도 |
| AI5002 | 500 | AI 연결 실패 | OpenAI 타임아웃 | 잠시 후 재시도 |
| AI5031 | 503 | AI 서비스 불가 | OpenAI 서비스 장애 | 관리자 문의 |

```mermaid
flowchart TB
    ANALYZE["AI 분석 요청"]
    ANALYZE --> CHECK_DATA{"데이터<br/>충분?"}
    CHECK_DATA -->|No| AI4002["AI4002"]
    CHECK_DATA -->|Yes| CHECK_LIMIT{"월간 한도<br/>내?"}
    CHECK_LIMIT -->|No| AI4031["AI4031"]
    CHECK_LIMIT -->|Yes| CALL["OpenAI 호출"]
    CALL --> TIMEOUT{"타임아웃?"}
    TIMEOUT -->|Yes| AI5002["AI5002"]
    TIMEOUT -->|No| PARSE{"응답<br/>파싱?"}
    PARSE -->|Fail| AI5001["AI5001"]
    PARSE -->|OK| SUCCESS["분석 성공"]
```

---

## 🔍 검색 에러 (SEARCH)

| Code | HTTP | 설명 | 발생 상황 |
|------|------|------|----------|
| SEARCH4001 | 400 | 검색어 없음 | 빈 검색어 |
| SEARCH4002 | 400 | 잘못된 날짜 범위 | 시작일 > 종료일 |

---

## 📊 에러 코드 체계

### 코드 구조

```
PREFIX + STATUS + SEQUENCE
  │        │        │
  │        │        └── 순번 (2자리)
  │        └── HTTP 상태 (3자리)
  └── 도메인 접두사
```

### 예시

```
AUTH4001
  │  │ │
  │  │ └── 01 (첫 번째 에러)
  │  └── 4 (400번대 = 클라이언트 에러)
  └── AUTH (인증 도메인)

AI5002
  │ │ │
  │ │ └── 02 (두 번째 에러)
  │ └── 5 (500번대 = 서버 에러)
  └── AI (AI 도메인)
```

---

## 🎯 HTTP Status 매핑

| HTTP Status | 의미 | 대표 에러 |
|-------------|------|----------|
| 200 | 성공 | COMMON200, AUTH2001 |
| 400 | 잘못된 요청 | COMMON400, RETRO4001 |
| 401 | 인증 필요 | AUTH4001, AUTH4004 |
| 403 | 권한 없음 | RETRO4031, AI4031 |
| 404 | 리소스 없음 | RETRO4041, RES4041 |
| 409 | 충돌 | AUTH4091, RETRO4091 |
| 500 | 서버 에러 | AI5001, AI5002 |
| 503 | 서비스 불가 | AI5031 |

---

## 🔧 에러 응답 형식

### 표준 응답

```json
{
  "isSuccess": false,
  "code": "RETRO4031",
  "message": "회고방 수정 권한이 없습니다",
  "result": null
}
```

### 검증 에러 (상세)

```json
{
  "isSuccess": false,
  "code": "COMMON400",
  "message": "입력값 검증에 실패했습니다",
  "result": {
    "errors": [
      {
        "field": "nickname",
        "message": "닉네임은 필수입니다"
      }
    ]
  }
}
```

---

## 🔗 Related

- [[00-HOME|🏠 HOME]]
- [[01-Architecture|🏗️ Architecture]]
- [[05-API-Overview|🔌 API Overview]]

---

#error #code #reference
