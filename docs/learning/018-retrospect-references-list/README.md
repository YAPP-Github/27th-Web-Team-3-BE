# API-018: 회고 참고자료 목록 조회

## 개요

특정 회고에 등록된 모든 참고자료(URL) 목록을 조회하는 API입니다.
회고 생성 시 등록했던 외부 링크들을 확인할 수 있으며, 등록 순서(referenceId 오름차순)로 정렬되어 반환됩니다.

## 엔드포인트

```
GET /api/v1/retrospects/{retrospectId}/references
```

## 주요 파일

| 파일 | 역할 | 핵심 함수/구조체 |
|------|------|------------------|
| `codes/server/src/domain/retrospect/handler.rs` | HTTP 핸들러 | `list_references` (라인 181~203) |
| `codes/server/src/domain/retrospect/service.rs` | 비즈니스 로직 | `list_references` (라인 430~458) |
| `codes/server/src/domain/retrospect/dto.rs` | 응답 DTO | `ReferenceItem` (라인 251~260) |
| `codes/server/src/domain/retrospect/entity/retro_reference.rs` | 엔티티 모델 | `Model` (라인 4~12) |
| `docs/api-specs/018-retrospect-references-list.md` | API 명세서 | 전체 스펙 정의 |

## API 특징

- **인증 필수**: Bearer 토큰 기반 JWT 인증
- **권한 검증**: 회고가 속한 팀의 멤버만 조회 가능
- **보안 처리**: 비멤버에게 회고 존재 여부를 노출하지 않도록 404 통합 처리
- **빈 결과 허용**: 참고자료가 없는 경우 빈 배열(`[]`) 반환
- **단순 조회**: 페이지네이션 없이 전체 목록을 한 번에 반환

## 스펙 vs 구현 차이점

API 명세서(`docs/api-specs/018-retrospect-references-list.md`)와 실제 구현 사이에 다음과 같은 차이가 있습니다.

| 항목 | API 명세서 | 실제 구현 | 비고 |
|------|-----------|-----------|------|
| 403 Forbidden (TEAM4031) | 별도 에러로 정의됨 | **반환되지 않음** | 구현에서는 멤버가 아닌 경우에도 404(RETRO4041)를 반환. `find_retrospect_for_member` 헬퍼가 보안상 403과 404를 통합 처리 |
| urlName 필드 설명 | "자료 별칭 (예: 깃허브 레포지토리)" | URL 원본 값이 그대로 들어감 | 회고 생성 시 `title: Set(url.clone())`으로 저장하므로 `urlName`과 `url`이 항상 동일한 값 (service.rs:162) |
| urlName 최대 길이 | 최대 50자 | URL 길이 제약(최대 2,048자)만 적용 | title 필드에 URL을 그대로 복사하므로, 별도의 50자 제약은 없음 |
| 에러 메시지 (404) | "존재하지 않는 회고 세션입니다." | "존재하지 않는 회고이거나 접근 권한이 없습니다." | 구현에서는 회고 미존재와 접근 권한 없음을 동일한 메시지로 통합 |

## 관련 학습 문서

- [flow.md](./flow.md) - 동작 흐름 상세 설명
- [key-concepts.md](./key-concepts.md) - 핵심 개념 정리
- [keywords.md](./keywords.md) - 학습 키워드
- [spring-guide.md](./spring-guide.md) - Spring 개발자를 위한 가이드 사전
