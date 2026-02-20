# [#120] 회고 생성 시 질문 커스텀 기능 추가

## 개요

회고 생성 API(`POST /api/v1/retrospects`)에 `questions` 필드를 필수로 추가하여,
프론트엔드에서 질문 목록을 직접 전달받아 저장하도록 변경했습니다.

## 변경 사항

### 1. API 스펙 (v2.0.0)
- `docs/api-specs/012-retrospect-create.md` 버전 2.0.0 업데이트
- `questions: Vec<String>` 필드 추가 (필수, 최소 1개, 빈 문자열 불가)
- 기존 "회고 방식별 기본 질문 생성 로직" 섹션을 "questions 검증 규칙"으로 대체
- 에러 코드 및 에러 응답 추가

### 2. DTO (`dto.rs`)
- `CreateRetrospectRequest`에 `questions: Vec<String>` 필드 추가
- `validate_question_items()` 커스텀 검증 함수 추가 (빈 문자열/공백 검증)

### 3. Entity (`retrospect.rs`)
- `retrospects` 테이블에 `questions: Option<String>` JSON 컬럼 추가
- `question_count()` 메서드를 `#[cfg(test)]`로 제한 (테스트 전용)

### 4. Service (`service.rs`)
- `get_questions_from_retrospect()` 헬퍼 메서드 추가
  - `questions` JSON 필드가 있으면 파싱, 없으면 기본 질문 폴백 (하위 호환)
- `create_retrospect`: 질문을 JSON으로 직렬화하여 저장
- `create_participant`: `default_questions()` 대신 저장된 질문 사용
- `save_draft`: 저장된 질문 수 기반 검증
- `submit_retrospect`: 저장된 질문 수 기반 검증
- `generate_assistant_guide`: 저장된 질문 기반 질문 내용 조회

### 5. DB 마이그레이션 (`database.rs`)
- `retrospects` 테이블에 `questions TEXT NULL` 컬럼 추가

### 6. 테스트
- questions 검증 테스트 4개 추가:
  - 빈 배열 → 실패
  - 빈 문자열 항목 → 실패
  - 공백만 있는 항목 → 실패
  - 유효한 질문 → 성공

## 하위 호환성

- 기존 회고(questions 컬럼이 NULL인 경우)는 `default_questions()` 폴백으로 정상 동작
- DB 마이그레이션은 `TEXT NULL`로 기존 데이터에 영향 없음

## 체크리스트

- [x] `cargo test` 통과
- [x] `cargo clippy -- -D warnings` 경고 없음
- [x] `cargo fmt` 적용
- [x] API 스펙 문서 업데이트
- [x] 리뷰 문서 작성
