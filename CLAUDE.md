# Web-3 Backend Project (Rust)

## 프로젝트 개요

**회고록 작성을 도와주는 AI 서비스**의 Rust 백엔드입니다.

## 기술 스택

- **언어**: Rust 1.84+
- **프레임워크**: Axum
- **Async Runtime**: Tokio
- **AI**: async-openai (OpenAI API)
- **검증**: validator
- **로깅**: tracing
- **문서화**: utoipa (OpenAPI)

## 디렉토리 구조

```
27th-Web-Team-3-BE/
├── .github/                    # GitHub Actions CI/CD
├── .claude/                    # Claude Code 설정
│   ├── commands/               # Slash 명령어 (/build, /test, /quality)
│   ├── hooks/                  # 자동화 훅
│   ├── rules/                  # 코딩 규칙
│   └── skills/                 # AI 스킬
├── codes/                      # Rust 소스 코드
│   └── server/                 # 백엔드 서버
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs
│       │   ├── config.rs       # 환경 설정
│       │   ├── error.rs        # 에러 타입
│       │   ├── response.rs     # 공통 응답
│       │   ├── utils/          # 공통 유틸리티
│       │   │   ├── mod.rs
│       │   │   ├── error.rs    # AppError 정의
│       │   │   └── response.rs # API 응답 구조체
│       │   ├── domain/
│       │   │   └── ai/
│       │   │       ├── handler.rs   # API 핸들러
│       │   │       ├── service.rs   # 비즈니스 로직
│       │   │       ├── dto.rs       # Request/Response
│       │   │       └── prompt.rs    # 프롬프트 템플릿
│       │   └── global/
│       │       └── middleware.rs
│       └── tests/              # 통합 테스트
├── docs/                       # 문서
│   ├── api-specs/              # API 명세서
│   ├── reviews/                # API 구현 리뷰 문서
│   ├── implementations/        # 구현 설명서
│   ├── meetings/               # 회의록
│   ├── requirements/           # 요구사항
│   └── ai-conventions/         # AI 협업 가이드
│       ├── claude.md           # Rust 코딩 규칙
│       └── architecture.md     # 아키텍처 설명
└── CLAUDE.md                   # 이 파일
```

## 빠른 시작

### 환경 설정
```bash
# Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add rustfmt clippy

# 환경 변수 설정
cp codes/server/.env.example codes/server/.env
# .env 파일 편집하여 API 키 설정
```

### 빌드 및 실행
```bash
cd codes/server

# 빌드
cargo build

# 실행
cargo run

# 릴리즈 빌드
cargo build --release
```

### 테스트
```bash
cd codes/server

# 모든 테스트
cargo test

# 특정 테스트
cargo test test_name

# 출력 포함
cargo test -- --nocapture
```

### 코드 품질
```bash
cd codes/server

# 포맷팅
cargo fmt

# 린트 (경고를 에러로)
cargo clippy -- -D warnings
```

## API 스펙

### POST /api/ai/retrospective/guide
회고 작성 가이드 제공

**Request**
```json
{
  "currentContent": "오늘 프로젝트를 진행하면서...",
  "secretKey": "your-secret-key"
}
```

**Response (200)**
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "currentContent": "오늘 프로젝트를 진행하면서...",
    "guideMessage": "좋은 시작이에요! ..."
  }
}
```

### POST /api/ai/retrospective/refine
회고 말투 정제

**Request**
```json
{
  "content": "오늘 일 힘들었음",
  "toneStyle": "KIND",
  "secretKey": "your-secret-key"
}
```

**Response (200)**
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "originalContent": "오늘 일 힘들었음",
    "refinedContent": "오늘 일이 많이 힘들었어요.",
    "toneStyle": "KIND"
  }
}
```

## 에러 코드

| 코드 | HTTP | 설명 |
|------|------|------|
| COMMON400 | 400 | 잘못된 요청 (필드 누락 등) |
| COMMON500 | 500 | 서버 내부 에러 |

- 필요한 에러 코드는 이후에 추가 작성

## 개발 원칙

### TDD (Test-Driven Development)
1. **Red**: 실패하는 테스트 먼저 작성
2. **Green**: 테스트 통과하는 최소 코드 구현
3. **Refactor**: 코드 개선

### Tidy First
- 구조적 변경과 행동적 변경을 분리
- 구조적 변경을 먼저 수행
- 같은 커밋에 두 유형을 섞지 않음

### 커밋 전 체크리스트
- [ ] `cargo test` 통과
- [ ] `cargo clippy -- -D warnings` 경고 없음
- [ ] `cargo fmt` 적용

## AI 협업 가이드

### 핵심 규칙 (반드시 준수)

| 규칙 | 설명 |
|------|------|
| **에러 처리** | `unwrap()` / `expect()` 금지, `Result` + `?` 사용 |
| **네이밍** | 함수: `snake_case`, 타입: `PascalCase` |
| **직렬화** | `#[serde(rename_all = "camelCase")]` 필수 |
| **테스트** | AAA 패턴 (Arrange-Act-Assert) |

### 상세 가이드 참조
- [docs/ai-conventions/claude.md](docs/ai-conventions/claude.md) - Rust 코딩 규칙
- [docs/ai-conventions/architecture.md](docs/ai-conventions/architecture.md) - 아키텍처 설명

## Claude Code 자동화

### Slash Commands
| 명령어 | 설명 |
|--------|------|
| `/build` | 포맷팅 → 린트 → 빌드 |
| `/test` | 테스트 실행 및 분석 |
| `/quality` | 전체 품질 검사 |
| `/run` | 서버 실행 |

### Rules 적용 범위
| Rule | 적용 경로 | 핵심 내용 |
|------|----------|----------|
| `rust-src` | `codes/server/src/**/*.rs` | 에러 처리, 로깅, 직렬화 |
| `rust-tests` | `codes/server/tests/**/*.rs` | 테스트 구조, AAA 패턴 |
| `api-design` | `handler.rs`, `dto.rs` | 응답 형식, 검증 |

### 자동화 워크플로우
```
[코드 작성] → [cargo fmt 자동 적용] → [검증]
     ↓
[/build 또는 /test 실행]
     ↓
[clippy 경고 확인]
     ↓
[커밋 전 체크리스트 확인]
```

## 작업 순서

1. 공통 유틸리티 확인 (`src/utils`)
2. 전체 API 테스트 및 green 테스트 작성
3. 구현
4. 전체 테스트 실행 및 검증 (서버 실행 후 `/health` 체크가 되면 그 다음 진행)
5. `docs/reviews/{api_name}.md` 문서 작성
6. 코드 리뷰 체크리스트 확인

## 코드 리뷰 체크리스트

- [ ] TDD 원칙을 따라 테스트 코드가 먼저 작성되었는가?
- [ ] 모든 테스트가 통과하는가?
- [ ] API 문서가 `docs/reviews/` 디렉토리에 작성되었는가?
- [ ] 공통 유틸리티를 재사용했는가?
- [ ] 에러 처리가 적절하게 되어 있는가?
- [ ] 코드가 Rust 컨벤션을 따르는가?
- [ ] 불필요한 의존성이 추가되지 않았는가?

## API 문서 작성 예시

---

# 📝 API 작업 리포트

## 1. 요약 (Summary)

### 💡 고려한 요구 사항

* **기능 핵심**: [예: 사용자 프로필 정보의 정합성 유지 및 보안 마스킹 적용]
* **예외 케이스**: [예: 탈퇴 계정 접근 차단, 잘못된 ID 형식에 대한 유효성 검사]
* **기타 고려사항**: [예: 응답 속도 최적화를 위한 인덱스 활용, 캐시 무효화 로직]

### 📤 요청 (Request)

* **Method**: `[GET / POST / PUT / DELETE]`
* **Endpoint**: `[주소 예: /api/v1/resource]`
* **Auth**: `[JWT / API Key / None]`

### 📥 응답 리스트 (Response List)

* **200 OK**: 정상적으로 데이터를 처리하고 결과를 반환함
* **400 Bad Request**: 필수 파라미터 누락 또는 데이터 형식이 올바르지 않음
* **401 Unauthorized**: 인증 토큰이 없거나 유효하지 않음
* **404 Not Found**: 요청한 자원을 찾을 수 없거나 삭제된 상태임

### ✅ 수행한 테스트 (Checklist)

* [ ] [테스트 항목 1: 정상 시나리오 검증]
* [ ] [테스트 항목 2: 빈 값/null 입력 시 에러 처리]
* [ ] [테스트 항목 3: 권한 없는 사용자의 접근 차단]
* [ ] [테스트 항목 4: 데이터베이스 반영 여부 확인]

---

## 2. 상세 내역 (Detailed Details)

### 📋 자세한 요구사항

1. **[구체적 요구사항 명칭]**: [해당 기능이 비즈니스적으로 어떻게 동작해야 하는지 상세히 기술합니다.]
2. **[데이터 제약 조건]**: [필드별 최대 길이, 필수 여부, 데이터 타입 등의 제약 사항을 기술합니다.]
3. **[비즈니스 로직]**: [단순 CRUD 외에 내부적으로 처리되는 계산이나 상태 변경 로직을 기술합니다.]

### 🛠 실제 요청/응답 예시 (JSON)

#### **실제 요청 (Request JSON)**

```json
{
  "example_key": "example_value",
  "data": {
    "id": 1,
    "name": "test_user"
  }
}

```

#### **실제 응답 (Response JSON)**

```json
{
  "status": "success",
  "code": 200,
  "data": {
    "id": 1,
    "name": "test_user",
    "created_at": "2026-01-17T15:30:00Z"
  },
  "message": "요청이 성공적으로 처리되었습니다."
}

```

### 🧪 수행한 테스트 상세

1. **[테스트 명칭]**:
* **방법**: [어떤 툴(Postman, curl 등)을 사용하여 어떤 데이터를 보냈는지 기록]
* **결과**: [예상한 응답 코드와 실제 데이터가 일치했는지 기술]


2. **[예외 상황 테스트]**:
* **방법**: [비정상적인 데이터를 주입하거나 강제로 에러 상황을 유도함]
* **결과**: [정의된 에러 메시지와 상태 코드가 정확히 반환되었는지 확인]

---


## 참고 자료

- [docs/api-specs/](docs/api-specs/) - API 상세 스펙
- [docs/ai-conventions/](docs/ai-conventions/) - AI 협업 가이드
