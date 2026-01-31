# API 학습 노트

구현된 API들을 기술적으로 분석하고 학습한 내용을 정리하는 공간입니다.
API 번호는 `docs/api-specs/`의 번호 체계와 동일합니다.

## 폴더 구조

각 API 폴더에는 다음 문서가 포함됩니다:

| 파일 | 내용 |
|------|------|
| `README.md` | API 개요, 엔드포인트, 관련 소스 파일 |
| `flow.md` | Handler → Service → DB 동작 흐름 |
| `key-concepts.md` | 핵심 Rust/Axum/SeaORM 패턴 분석 |
| `keywords.md` | 학습 키워드 정리 + 코드 위치 참조 |

## API 목록

| # | API | 폴더 | 문서 |
|---|-----|------|:----:|
| 011 | 팀 회고 목록 조회 | [`011-team-retrospects-list/`](./011-team-retrospects-list/) | 4개 |
| 012 | 회고 생성 | [`012-retrospect-create/`](./012-retrospect-create/) | 1개 |
| 013 | 회고 상세 조회 | [`013-retrospect-detail/`](./013-retrospect-detail/) | 4개 |
| 014 | 회고 삭제 | [`014-retrospect-delete/`](./014-retrospect-delete/) | 4개 |
| 015 | 회고 참여자 등록 | [`015-retrospect-participant-create/`](./015-retrospect-participant-create/) | 4개 |
| 017 | 답변 임시 저장 | [`017-retrospect-draft-save/`](./017-retrospect-draft-save/) | 4개 |
| 018 | 답변 제출 | [`018-retrospect-submit/`](./018-retrospect-submit/) | 4개 |
| 019 | 참고자료 목록 | [`019-retrospect-references-list/`](./019-retrospect-references-list/) | 4개 |
| 020 | 보관함 조회 | [`020-retrospect-storage/`](./020-retrospect-storage/) | 4개 |
| 021 | 답변 카테고리별 조회 | [`021-retrospect-responses-list/`](./021-retrospect-responses-list/) | 4개 |
| 022 | 회고 내보내기 (PDF) | [`022-retrospect-export/`](./022-retrospect-export/) | 4개 |
| 023 | 회고 AI 분석 | - [023-retrospect-analysis](023-retrospect-analysis/spring-guide.md) | 4개 |
| 024 | 회고 검색 | [`024-retrospect-search/`](./024-retrospect-search/) | 4개 |

## 참고 자료
- [API 스펙 문서](../api-specs/)
- [구현 리뷰 문서](../reviews/)
- [아키텍처 가이드](../ai-conventions/architecture.md)
- [Rust 코딩 컨벤션](../ai-conventions/claude.md)
