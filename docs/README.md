# Docs - 문서 디렉토리

프로젝트의 모든 문서를 포함합니다. AI 에이전트가 컨텍스트를 파악하는 데 활용됩니다.

## 구조

```
docs/
├── api-specs/          # API 명세서 (OpenAPI, Swagger)
├── reviews/            # API 구현 리뷰 문서
├── implementations/    # 구현 설명서 (PR Description 대용)
├── meetings/           # 회의록
├── requirements/       # 기능 요구사항
└── ai-conventions/     # AI 협업 가이드
    ├── claude.md       # Rust 코딩 규칙
    ├── architecture.md # 아키텍처 설명
    └── ai_collaboration_guide.md # AI 협업 시스템 설명
```

## 폴더별 용도

### api-specs/
- API 엔드포인트 명세
- Request/Response 스키마
- 에러 코드 정의
- 총 27개 API 명세 포함

### reviews/
- API 구현 리뷰 문서
- 각 API별 구현 완료 후 작성하는 리뷰
- 코드 리뷰 체크리스트 포함

### implementations/
- 각 기능 구현에 대한 설명
- PR 설명 대신 여기에 문서화
- AI가 구현 맥락을 파악하는 데 활용

### meetings/
- 팀 회의록
- 의사결정 기록

### requirements/
- 기능 요구사항
- 사용자 스토리
- 우선순위 정보

### ai-conventions/
- AI 에이전트를 위한 코딩 가이드라인
- 아키텍처 설명
- 코드 스타일 규칙
