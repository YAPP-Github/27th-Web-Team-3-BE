# 👤 User Scenarios

> 주요 사용자 시나리오별 API 호출 흐름

---

## 📍 Overview

```mermaid
journey
    title 서비스 전체 여정
    section 시작
      앱 설치: 5: User
      소셜 로그인: 5: User
    section 팀 설정
      팀 생성: 5: Leader
      초대 코드 공유: 4: Leader
      팀 합류: 5: Member
    section 회고 진행
      회고 생성: 5: Leader
      참석 등록: 4: Member
      답변 작성: 4: Member
      최종 제출: 5: Member
    section 분석
      AI 분석: 5: Leader
      결과 확인: 5: All
    section 공유
      댓글/좋아요: 4: Member
      PDF 내보내기: 4: Leader
```

---

## 🔐 시나리오 1: 신규 사용자 가입

### 흐름

```mermaid
sequenceDiagram
    participant U as 사용자
    participant A as App
    participant S as Server
    participant K as Kakao/Google

    U->>A: 앱 실행
    A->>U: 로그인 화면

    U->>K: 소셜 로그인 선택
    K-->>A: 소셜 액세스 토큰

    A->>S: API-001 소셜 로그인
    S-->>A: isNewMember: true, signupToken

    A->>U: 닉네임 입력 화면
    U->>A: 닉네임 입력

    A->>S: API-002 회원가입
    S-->>A: accessToken, refreshToken

    A->>U: 메인 화면
```

### API 호출 순서

| 순서 | API | 설명 |
|:----:|-----|------|
| 1 | API-001 | 소셜 로그인 → signupToken 받음 |
| 2 | API-002 | 회원가입 → 토큰 발급 |

---

## 👑 시나리오 2: 팀 리더 - 회고방 생성

### 흐름

```mermaid
sequenceDiagram
    participant L as 팀 리더
    participant A as App
    participant S as Server

    L->>A: 새 회고방 버튼
    A->>L: 회고방 정보 입력 폼

    L->>A: 이름, 설명 입력
    A->>S: API-005 회고방 생성
    S-->>A: retroRoomId, invitationUrl

    A->>L: 초대 코드 표시
    Note over L: INV-A1B2-C3D4

    L-->>L: 팀원에게 코드 공유
```

### API 호출 순서

| 순서 | API | 설명 |
|:----:|-----|------|
| 1 | API-005 | 회고방 생성 |
| - | - | 초대 코드 공유 (앱 외부) |

---

## 👥 시나리오 3: 팀원 - 팀 합류

### 흐름

```mermaid
sequenceDiagram
    participant M as 팀원
    participant A as App
    participant S as Server

    M->>A: 초대 코드 입력
    Note over M: INV-A1B2-C3D4

    A->>S: API-006 팀 합류
    S-->>A: retroRoom 정보

    A->>S: API-007 팀 목록
    S-->>A: 내 회고방 목록

    A->>M: 회고방 목록 화면
```

### API 호출 순서

| 순서 | API | 설명 |
|:----:|-----|------|
| 1 | API-006 | 초대 코드로 팀 합류 |
| 2 | API-007 | 팀 목록 갱신 |

---

## 📝 시나리오 4: 회고 생성 및 진행

### 흐름 (리더)

```mermaid
sequenceDiagram
    participant L as 팀 리더
    participant A as App
    participant S as Server

    L->>A: 새 회고 버튼
    A->>L: 회고 설정 폼

    L->>A: 제목, 방식, 참고자료 입력
    A->>S: API-012 회고 생성
    S-->>A: retrospectId

    A->>L: 회고 생성 완료
```

### 흐름 (참여자)

```mermaid
sequenceDiagram
    participant M as 팀원
    participant A as App
    participant S as Server

    A->>S: API-011 회고 목록
    S-->>A: 회고 목록

    M->>A: 회고 선택
    A->>S: API-013 회고 상세
    S-->>A: 회고 정보, 질문 목록

    M->>A: 참석 버튼
    A->>S: API-015 참석 등록
    S-->>A: 성공

    Note over M: 답변 작성 시작

    loop 작성 중
        M->>A: 답변 입력
        A->>S: API-016 임시 저장
        S-->>A: 성공
    end

    M->>A: 제출 버튼
    A->>S: API-017 최종 제출
    S-->>A: 성공
```

### API 호출 순서 (리더)

| 순서 | API | 설명 |
|:----:|-----|------|
| 1 | API-012 | 회고 생성 |

### API 호출 순서 (참여자)

| 순서 | API | 설명 |
|:----:|-----|------|
| 1 | API-011 | 회고 목록 조회 |
| 2 | API-013 | 회고 상세 조회 |
| 3 | API-015 | 참석 등록 |
| 4 | API-016 | 임시 저장 (반복) |
| 5 | API-017 | 최종 제출 |

---

## 🤖 시나리오 5: AI 분석 및 결과 확인

### 흐름

```mermaid
sequenceDiagram
    participant L as 팀 리더
    participant A as App
    participant S as Server
    participant AI as OpenAI

    L->>A: AI 분석 버튼
    A->>S: API-022 AI 분석
    S->>AI: 분석 요청
    AI-->>S: 분석 결과
    S-->>A: teamInsight, emotionRank, missions

    A->>L: 분석 결과 화면

    L->>A: 카테고리별 보기
    A->>S: API-020 카테고리별 조회
    S-->>A: 답변 목록

    A->>L: 답변 및 인사이트 표시
```

### API 호출 순서

| 순서 | API | 설명 |
|:----:|-----|------|
| 1 | API-022 | AI 분석 요청 |
| 2 | API-020 | 카테고리별 답변 조회 |

---

## 💬 시나리오 6: 소셜 상호작용

### 흐름

```mermaid
sequenceDiagram
    participant M as 팀원
    participant A as App
    participant S as Server

    A->>S: API-020 카테고리별 조회
    S-->>A: 답변 목록

    M->>A: 답변 확인

    M->>A: 좋아요 버튼
    A->>S: API-025 좋아요 토글
    S-->>A: isLiked: true

    M->>A: 댓글 버튼
    A->>S: API-026 댓글 조회
    S-->>A: 댓글 목록

    M->>A: 댓글 작성
    A->>S: API-027 댓글 작성
    S-->>A: 새 댓글
```

### API 호출 순서

| 순서 | API | 설명 |
|:----:|-----|------|
| 1 | API-020 | 답변 조회 |
| 2 | API-025 | 좋아요 토글 |
| 3 | API-026 | 댓글 조회 |
| 4 | API-027 | 댓글 작성 |

---

## 📤 시나리오 7: PDF 내보내기

### 흐름

```mermaid
sequenceDiagram
    participant U as 사용자
    participant A as App
    participant S as Server

    U->>A: 내보내기 버튼
    A->>S: API-021 PDF 내보내기
    S->>S: PDF 생성
    S-->>A: PDF 파일

    A->>U: 파일 다운로드
    Note over U: retrospect_20240115_120000.pdf
```

### API 호출 순서

| 순서 | API | 설명 |
|:----:|-----|------|
| 1 | API-021 | PDF 내보내기 |

---

## 🔍 시나리오 8: 회고 검색

### 흐름

```mermaid
sequenceDiagram
    participant U as 사용자
    participant A as App
    participant S as Server

    U->>A: 검색 화면
    U->>A: 키워드 입력 + 필터

    A->>S: API-023 검색
    Note over A,S: keyword, method, dateRange

    S-->>A: 검색 결과

    U->>A: 결과 선택
    A->>S: API-013 회고 상세
    S-->>A: 회고 정보
```

### API 호출 순서

| 순서 | API | 설명 |
|:----:|-----|------|
| 1 | API-023 | 회고 검색 |
| 2 | API-013 | 회고 상세 조회 |

---

## 🚪 시나리오 9: 로그아웃 및 탈퇴

### 로그아웃

```mermaid
sequenceDiagram
    participant U as 사용자
    participant A as App
    participant S as Server

    U->>A: 로그아웃 버튼
    A->>S: API-004 로그아웃
    S-->>A: 성공

    A->>A: 토큰 삭제
    A->>U: 로그인 화면
```

### 서비스 탈퇴

```mermaid
sequenceDiagram
    participant U as 사용자
    participant A as App
    participant S as Server

    U->>A: 탈퇴 버튼
    A->>U: 확인 다이얼로그

    U->>A: 탈퇴 확인
    A->>S: API-028 서비스 탈퇴
    S-->>A: 성공

    A->>A: 토큰 삭제
    A->>U: 로그인 화면
```

---

## 📊 시나리오별 API 요약

| 시나리오 | API 수 | 주요 API |
|---------|:------:|---------|
| 신규 가입 | 2 | 001, 002 |
| 팀 생성 | 1 | 005 |
| 팀 합류 | 2 | 006, 007 |
| 회고 진행 | 5 | 011-017 |
| AI 분석 | 2 | 022, 020 |
| 소셜 | 3 | 025-027 |
| PDF | 1 | 021 |
| 검색 | 2 | 023, 013 |
| 로그아웃 | 1 | 004 |
| 탈퇴 | 1 | 028 |

---

## 🔗 Related

- [[00-HOME|🏠 HOME]]
- [[05-API-Overview|🔌 API Overview]]
- [[03-Retrospect-Flow|📝 Retrospect Flow]]

---

#scenario #user #flow #journey
