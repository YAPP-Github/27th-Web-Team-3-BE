# Web-3 Backend Project (Rust)

## 프로젝트 개요
회고록 작성을 도와주는 AI 서비스의 Rust 백엔드입니다.

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
|       |   |── config/
|       |   |   └── mod.rs      # 설정 모듈
|       │   |   └── app_config.rs # 애플리케이션 설정
|       |   |   └── database.rs   # 데이터베이스 설정
│       │   ├── main.rs
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

## 작업 순서

1. 공통 유틸리티 확인 (`src/utils`), .env.example 확인
2. 전체 API 테스트 및 green 테스트 작성
3. 구현 코드 작성 (스웨거 작성은 아직 하지 않음)
4. 같은 dto 에 스웨거용 전체 dto 만들고 핸들러에 스웨거 작성 
5. 전체 테스트 실행 및 검증 (서버 실행 후 `/health` 체크가 되면 그 다음 진행)
6. `docs/reviews/{api_name}.md` 문서 작성
7. 코드 리뷰 체크리스트 확인

## 코드 리뷰 체크리스트

- [ ] TDD 원칙을 따라 테스트 코드가 먼저 작성되었는가?
- [ ] 모든 테스트가 통과하는가?
- [ ] API 문서가 `docs/reviews/` 디렉토리에 작성되었는가?
- [ ] 공통 유틸리티를 재사용했는가?
- [ ] 에러 처리가 적절하게 되어 있는가?
- [ ] 코드가 Rust 컨벤션을 따르는가?
- [ ] 불필요한 의존성이 추가되지 않았는가?

## 참고 자료

- [docs/api-specs/](docs/api-specs/) - API 상세 스펙
- [docs/ai-conventions/](docs/ai-conventions/) - AI 협업 가이드