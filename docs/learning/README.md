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
| 010 | 팀 회고 목록 조회 | [`010-team-retrospects-list/`](./010-team-retrospects-list/) | 4개 |
| 011 | 회고 생성 | [`011-retrospect-create/`](./011-retrospect-create/) | 1개 |
| 012 | 회고 상세 조회 | [`012-retrospect-detail/`](./012-retrospect-detail/) | 4개 |
| 013 | 회고 삭제 | [`013-retrospect-delete/`](./013-retrospect-delete/) | 4개 |
| 014 | 회고 참여자 등록 | [`014-retrospect-participant-create/`](./014-retrospect-participant-create/) | 4개 |
| 016 | 답변 임시 저장 | [`016-retrospect-draft-save/`](./016-retrospect-draft-save/) | 4개 |
| 017 | 답변 제출 | [`017-retrospect-submit/`](./017-retrospect-submit/) | 4개 |
| 018 | 참고자료 목록 | [`018-retrospect-references-list/`](./018-retrospect-references-list/) | 4개 |
| 019 | 보관함 조회 | [`019-retrospect-storage/`](./019-retrospect-storage/) | 4개 |
| 020 | 답변 카테고리별 조회 | [`020-retrospect-responses-list/`](./020-retrospect-responses-list/) | 4개 |
| 021 | 회고 내보내기 (PDF) | [`021-retrospect-export/`](./021-retrospect-export/) | 4개 |
| 022 | 회고 AI 분석 | - [022-retrospect-analysis](022-retrospect-analysis/spring-guide.md) | 4개 |
| 023 | 회고 검색 | [`023-retrospect-search/`](./023-retrospect-search/) | 4개 |

## 참고 자료
- [API 스펙 문서](../api-specs/)
- [구현 리뷰 문서](../reviews/)
- [아키텍처 가이드](../ai-conventions/architecture.md)
- [Rust 코딩 컨벤션](../ai-conventions/claude.md)
