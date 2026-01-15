# 🤖 AI 협업 가이드 및 문서 시스템 (.claude)

## 1. 개요 (Overview)

이 문서는 **Web3 회고록 AI 서버 (Rust Migration)** 프로젝트에서 AI 에이전트(Claude 등)와 개발자가 효율적으로 협업하기 위해 구축된 문서 시스템의 구조와 상세 동작 원리를 설명합니다.

단순히 "코드를 짜줘"라고 요청하는 것을 넘어, **AI가 프로젝트의 컨텍스트(문맥)를 지속적으로 유지하고, Rust의 엄격한 아키텍처 규칙을 세션 내내 준수하도록 제어**하는 것이 이 시스템의 목표입니다.

---

## 2. 문서 시스템 구조 (System Structure)

프로젝트 루트와 `.claude` 디렉토리에 위치한 파일들은 AI의 **페르소나(Persona)**, **지식(Knowledge)**, **행동(Action)**을 정의하는 계층적 구조를 가집니다.

```text
(Root)
├── CLAUDE.md            # [Map] 프로젝트 진입점, 기술 스택 요약, 네비게이션 인덱스
├── agent.md             # [Persona] AI의 역할(Strict Teacher) 및 사고방식 정의
└── .claude/             # [Knowledge Base]
    ├── commands/        # [Tools] 실행 가능한 쉘 명령어 모음 (Run, Test, Build)
    ├── hooks/           # [Triggers] 작업 단계별 강제 확인 절차 (Context Anchor)
    ├── rules/           # [Constraints] 반드시 지켜야 할 코딩/아키텍처 규약
    └── skills/          # [Procedures] 복잡한 작업(리팩토링 등)의 표준 절차
```

---

## 3. 상세 구성 요소별 역할

### 3.1. 루트 파일: 세션의 기준점 (Anchor)

#### 📄 `CLAUDE.md` (Context Map)
*   **역할:** 프로젝트의 "여권"과 같습니다. 어떤 기술을 쓰는지(Rust/Axum), 어디에 무엇이 있는지 알려줍니다.
*   **동작 시점:** **세션 시작 직후.** AI는 이 파일을 통해 프로젝트의 전체 윤곽을 파악하고, 세부 규칙이 필요할 때 어디를(`rules/`) 봐야 할지 알게 됩니다.

#### 🧠 `agent.md` (System Persona)
*   **역할:** AI의 "자아"를 설정합니다. 여기서는 **"Strict Rust Architect"**로 설정되어 있습니다.
*   **동작 방식:** 이 파일이 컨텍스트에 로드되면, AI는 `unwrap()` 남발이나 불명확한 에러 처리를 스스로 검열합니다. 개발자가 대충 요청해도 "Rust의 안전성을 위해 Result 타입을 써야 합니다"라고 역제안하게 만듭니다.

### 3.2. `.claude` 디렉토리: 지식과 규율

#### 🛠️ `commands/` (Executable Knowledge)
*   **내용:** `run.md`, `check.md`, `doc.md` 등.
*   **동작:** "서버 띄워줘"라는 자연어 요청을 `cargo run`이라는 구체적인 명령어로 매핑합니다. 옵션(`--release` 등)까지 정확히 선택하게 돕습니다.

#### ⚓ `hooks/` (Safety Interceptors)
*   **내용:** `architecture-check.md` 등.
*   **동작 시점:** AI가 **파일을 생성(`write_file`)하거나 수정하기 직전**에 발동합니다.
*   **효과:** AI가 코드를 짜기 전에 "잠깐, 이 코드가 도메인 주도 설계(DDD)에 맞나?"라고 스스로 질문하게 만듭니다. 이는 AI의 충동적인 코딩을 막는 제동 장치(Brake)입니다.

#### 📏 `rules/` (Hard Constraints)
*   **내용:** API 설계 원칙, Rust 코딩 스타일 등.
*   **동작:** AI가 코드를 생성(Generation)할 때 참조하는 **제약 조건**입니다. 토큰 효율성을 위해 필요한 규칙만 선별적으로 읽힙니다.

---

## 4. 컨텍스트 관리 및 세션 동작 메커니즘 (Context & Session Dynamics)

AI와의 대화(Session)는 **유한한 컨텍스트 윈도우(Memory)**를 가집니다. 대화가 길어질수록 초기의 지시사항이 희미해지는(Context Drift) 현상이 발생합니다. 이 문서 시스템은 이를 방지하기 위해 **능동적 로딩 전략**을 사용합니다.

### 4.1. 세션 라이프사이클 (Session Lifecycle)

1.  **초기화 (Initialization):**
    *   **Action:** 사용자가 "프로젝트 파악해줘" 요청.
    *   **AI 동작:** `CLAUDE.md`와 `agent.md`를 읽음(`read_file`).
    *   **Result:** AI는 자신이 "Rust 전문가"임을 인지하고, 프로젝트 구조(DDD)를 머릿속에 인덱싱합니다.

2.  **작업 수행 및 온디맨드 로딩 (Execution & On-demand Loading):**
    *   **Action:** 사용자가 "API 하나 만들어줘" 요청.
    *   **AI 동작:**
        1.  `CLAUDE.md`를 참조하니 API 관련 규칙은 `.claude/rules/api-design.md`에 있다고 나옴.
        2.  **스스로 `read_file`을 호출**하여 해당 규칙 파일을 읽음. (전체 파일을 다 읽지 않고 필요한 것만 읽어 **토큰 절약**).
        3.  규칙을 숙지한 상태에서 코드 생성.

3.  **컨텍스트 유지 및 재확인 (Retention & Anchoring):**
    *   **Problem:** 대화가 20턴 이상 넘어가면 AI가 `agent.md`의 "Strict Mode"를 잊고 느슨한 코드를 짤 수 있음.
    *   **Solution (Hooks):** 코드를 파일에 쓰기 전, `.claude/hooks/architecture-check.md`가 트리거됨. AI는 다시 한번 아키텍처 원칙을 상기하고(Re-reading), 코드가 올바른 위치에 있는지 검증함.

### 4.2. 토큰 효율성 전략 (Token Efficiency)
모든 규칙 파일(`rules/*`)을 세션 시작 시 한 번에 다 읽으면 토큰 낭비가 심하고, AI가 중요하지 않은 정보에 집중할 수 있습니다.
*   **전략:** **Root Indexing + Lazy Loading**
*   `CLAUDE.md`는 가벼운 인덱스 역할만 수행합니다.
*   AI는 실제 작업이 주어졌을 때 관련된 상세 규칙 파일만 `read_file`로 가져와서 컨텍스트에 일시적으로 올립니다.

---

## 5. 실전 활용 시나리오 (Workflow Example)

개발자가 **"새로운 헬스 체크 API를 추가하고 싶어"**라고 요청했을 때, 내부적으로 어떤 문서들이 상호작용하는지 보여줍니다.

1.  **Trigger:** 사용자 요청 ("Add Health API")
2.  **Consult Map (`CLAUDE.md`):**
    *   AI: "API 추가 작업이군. `api-design.md` 규칙을 확인해야겠다."
3.  **Load Constraints (`read_file .claude/rules/api-design.md`):**
    *   AI: "규칙 확인: RESTful이어야 하고, 응답은 `BaseResponse` 포맷을 써야 하네."
4.  **Architectural Hook (`hooks/architecture-check.md`):**
    *   AI: (코드 작성 전) "체크리스트 확인. `health` 도메인이 이미 존재하나? `src/domain/health`에 `handler.rs`와 `service.rs`를 분리해야겠군."
5.  **Persona Enforcement (`agent.md`):**
    *   AI: "간단한 로직이지만 `unwrap`을 쓰면 안 돼. `agent.md`가 안전성을 강조했어. 에러 처리를 추가하자."
6.  **Action (`write_file`):**
    *   규칙과 아키텍처가 반영된 안전한 Rust 코드 작성.
7.  **Verify (`commands/check.md`):**
    *   AI: "이제 `cargo check` 명령어로 컴파일 에러가 없는지 검증하자."

---

## 6. 결론

이 문서 시스템은 AI를 단순한 "코드 생성기"가 아닌 **"프로젝트의 맥락을 이해하고 스스로 품질을 관리하는 동료"**로 격상시킵니다.

*   개발자는 AI에게 일일이 규칙을 설명할 필요가 없습니다. "문서대로 해" 한마디면 충분합니다.
*   AI는 `hooks`와 `rules`를 통해 스스로를 제어하며, 시간이 지나도 코드 스타일이 변질되지 않도록 `CLAUDE.md`를 기준으로 컨텍스트를 지속적으로 재조정(Realignment)합니다.