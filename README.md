# Moalog Backend

회고록 작성을 도와주는 AI 서비스 **Moalog**의 Rust 백엔드입니다.

## Overview

Moalog는 팀 회고를 효과적으로 진행할 수 있도록 도와주는 서비스입니다. AI 기반 분석과 어시스턴트 기능을 통해 회고의 질을 높이고, 팀 협업을 지원합니다.

### 주요 기능

- **소셜 로그인**: 카카오, 구글 OAuth 지원
- **회고방 관리**: 팀별 회고방 생성 및 초대 코드 기반 참여
- **회고 작성**: KPT, 4L, 5F 등 다양한 회고 방식 지원
- **AI 분석**: OpenAI 기반 회고 인사이트 및 맞춤형 미션 제공
- **AI 어시스턴트**: 회고 작성을 돕는 가이드 제공
- **PDF 내보내기**: 회고 내용을 PDF로 저장
- **댓글 & 좋아요**: 팀원 간 피드백 기능

## Tech Stack

| 분류 | 기술 |
|-----|------|
| **Language** | Rust (Edition 2021) |
| **Web Framework** | Axum 0.7 |
| **Async Runtime** | Tokio |
| **Database** | MySQL + SeaORM |
| **AI Integration** | OpenAI API (async-openai) |
| **Authentication** | JWT (jsonwebtoken) |
| **API Documentation** | OpenAPI + Swagger UI (utoipa) |
| **Logging** | Tracing |
| **Validation** | Validator |
| **PDF Generation** | genpdf |

## Project Structure

```
27th-Web-Team-3-BE/
├── codes/server/              # Rust 백엔드 서버
│   ├── src/
│   │   ├── main.rs            # 엔트리포인트 및 라우터
│   │   ├── config/            # 설정 (DB, 환경변수)
│   │   ├── utils/             # 공통 유틸 (에러, 응답, JWT)
│   │   ├── domain/            # 비즈니스 로직
│   │   │   ├── auth/          # 인증 (로그인, 회원가입)
│   │   │   ├── member/        # 회원 관리
│   │   │   ├── retrospect/    # 회고 (핵심 도메인)
│   │   │   ├── ai/            # AI 서비스
│   │   │   └── webhook/       # 웹훅 처리
│   │   ├── event/             # 이벤트 시스템
│   │   ├── monitoring/        # 로그 모니터링
│   │   └── automation/        # AI 자동화
│   └── tests/                 # 통합 테스트
├── docs/                      # 프로젝트 문서
│   ├── api-specs/             # API 명세서
│   ├── ai-conventions/        # 코딩 컨벤션
│   └── ai-monitoring/         # 모니터링 가이드
├── ci/                        # 인프라 (Terraform, Nginx)
└── scripts/                   # 자동화 스크립트
```

## Getting Started

### Prerequisites

- Rust 1.75+
- MySQL 8.0+
- OpenAI API Key (AI 기능 사용 시)

### Installation

```bash
# 저장소 클론
git clone https://github.com/YAPP-Github/27th-Web-Team-3-BE.git
cd 27th-Web-Team-3-BE/codes/server

# 환경변수 설정
cp .env.example .env
# .env 파일 수정

# 빌드
cargo build

# 실행
cargo run
```

### Environment Variables

```env
# Database
DATABASE_URL=mysql://localhost:3306/retrospect
DATABASE_USERNAME=root
DATABASE_PASSWORD=password

# Server
SERVER_PORT=8080

# JWT
JWT_SECRET=your-secret-key
JWT_EXPIRATION=3600
REFRESH_TOKEN_EXPIRATION=604800

# Social Login
GOOGLE_CLIENT_ID=your-google-client-id
KAKAO_CLIENT_ID=your-kakao-client-id

# AI (Optional)
OPENAI_API_KEY=sk-...
```

### Run Tests

```bash
cd codes/server

# 모든 테스트 실행
cargo test

# 특정 테스트
cargo test test_name

# 출력 포함
cargo test -- --nocapture
```

## API Endpoints

총 **30개** API 엔드포인트가 구현되어 있습니다.

### Authentication (6개)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/auth/social-login` | 소셜 로그인 (카카오/구글) |
| POST | `/api/v1/auth/signup` | 회원가입 완료 |
| POST | `/api/v1/auth/token/refresh` | 토큰 갱신 |
| POST | `/api/v1/auth/logout` | 로그아웃 |

### Retro Room (7개)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/retro-rooms` | 회고방 생성 |
| GET | `/api/v1/retro-rooms` | 회고방 목록 조회 |
| POST | `/api/v1/retro-rooms/join` | 회고방 참여 |
| PATCH | `/api/v1/retro-rooms/order` | 순서 변경 |
| PATCH | `/api/v1/retro-rooms/{id}/name` | 이름 변경 |
| DELETE | `/api/v1/retro-rooms/{id}` | 회고방 삭제 |
| GET | `/api/v1/retro-rooms/{id}/retrospects` | 회고 목록 |

### Retrospect (12개)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/retrospects` | 회고 생성 |
| GET | `/api/v1/retrospects/{id}` | 회고 상세 조회 |
| PUT | `/api/v1/retrospects/{id}/drafts` | 임시저장 |
| POST | `/api/v1/retrospects/{id}/submit` | 회고 제출 |
| DELETE | `/api/v1/retrospects/{id}` | 회고 삭제 |
| POST | `/api/v1/retrospects/{id}/analysis` | AI 분석 |
| GET | `/api/v1/retrospects/{id}/export` | PDF 내보내기 |
| GET | `/api/v1/retrospects/search` | 검색 |
| GET | `/api/v1/retrospects/storage` | 보관함 |

### Response & Comment (4개)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/retrospects/{id}/responses` | 답변 목록 |
| POST | `/api/v1/responses/{id}/comments` | 댓글 작성 |
| GET | `/api/v1/responses/{id}/comments` | 댓글 조회 |
| POST | `/api/v1/responses/{id}/likes` | 좋아요 토글 |

### API Documentation

서버 실행 후 Swagger UI에서 전체 API 문서를 확인할 수 있습니다:
- **Swagger UI**: `http://localhost:8080/swagger-ui/`

## Development

### Code Quality

```bash
cd codes/server

# 포맷팅
cargo fmt

# 린트 (경고를 에러로)
cargo clippy -- -D warnings

# 전체 품질 검사
cargo fmt && cargo clippy -- -D warnings && cargo test
```

### Commit Convention

| Gitmoji | Tag | Description |
|:-------:|:---:|-------------|
| ✨ | `feat` | 새로운 기능 추가 |
| 🐛 | `fix` | 버그 수정 |
| 📝 | `docs` | 문서 추가, 수정, 삭제 |
| ✅ | `test` | 테스트 코드 추가, 수정, 삭제 |
| 💄 | `style` | 코드 형식 변경 |
| ♻️ | `refactor` | 코드 리팩토링 |
| ⚡️ | `perf` | 성능 개선 |
| 💚 | `ci` | CI 관련 설정 수정 |
| 🚀 | `chore` | 기타 변경사항 |
| 🔥 | `remove` | 코드 및 파일 제거 |
| 🏗️ | `structure` | 구조적 변경 (Tidy First) |

### Coding Rules

- `unwrap()` / `expect()` 사용 금지 (테스트 제외)
- 모든 에러는 `Result<T, AppError>` 반환
- DTO에는 `#[serde(rename_all = "camelCase")]` 필수
- 테스트는 AAA 패턴 (Arrange-Act-Assert)

자세한 내용은 [docs/ai-conventions/claude.md](docs/ai-conventions/claude.md) 참조

## Documentation

| 문서 | 설명 |
|-----|------|
| [docs/api-specs/](docs/api-specs/) | API 상세 명세서 |
| [docs/ai-conventions/](docs/ai-conventions/) | 코딩 컨벤션 가이드 |
| [docs/ai-monitoring/](docs/ai-monitoring/) | AI 모니터링 파이프라인 |
| [docs/learning/](docs/learning/) | API 구현 학습 노트 |
| [CLAUDE.md](CLAUDE.md) | Claude Code 협업 가이드 |

## License

This project is private and proprietary to YAPP 27th Web Team 3.
