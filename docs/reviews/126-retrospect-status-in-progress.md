# #126 RetrospectStatus에 IN_PROGRESS 추가

## 변경 목적

참석자 등록 직후 `DRAFT` 상태로 설정되어 "아무것도 작성하지 않은 상태"와 "임시 저장한 상태"를 구분할 수 없는 문제를 해결합니다.

**상태 흐름**: `IN_PROGRESS` → `DRAFT` → `SUBMITTED` → `ANALYZED`

## 변경 파일

### 1. Entity 정의
- `codes/server/src/domain/member/entity/member_retro.rs`
  - `RetrospectStatus` enum에 `InProgress` variant 추가 (`string_value = "IN_PROGRESS"`)

### 2. DTO
- `codes/server/src/domain/retrospect/dto.rs`
  - `CurrentUserStatus` enum에 `InProgress` variant 추가

### 3. Service
- `codes/server/src/domain/retrospect/service.rs`
  - `create_participant()`: `status`를 `InProgress`로 명시적 설정
  - `save_draft()`: `InProgress` 상태일 때 `Draft`로 전환하는 로직 추가
  - `list_retrospects()`: `InProgress` → `RetrospectListStatus::InProgress` 매핑 추가
  - `get_retrospect_detail()`: `InProgress` → `CurrentUserStatus::InProgress` 매핑 추가
  - `generate_assistant_guide()`: `InProgress` 상태에서도 어시스턴트 사용 허용 (`!= Draft` → `== Submitted || == Analyzed`)

### 4. 변경하지 않은 부분
- `submit_retrospect()`: 기존에 `Submitted`/`Analyzed`만 차단하므로 `InProgress`에서도 제출 허용 (변경 불필요)
- `analyze_retrospective()`, `list_responses()`, `get_analysis_result()`: `Submitted`/`Analyzed` 필터 유지 (변경 불필요)

## DB 마이그레이션

PostgreSQL에서 아래 SQL 실행 필요:

```sql
ALTER TYPE "RetrospectStatus" ADD VALUE 'IN_PROGRESS' BEFORE 'DRAFT';
```

## 검증 결과

- `cargo build` 통과
- `cargo test` 전체 통과 (392 unit + 14 integration + 3 doc tests)
- `cargo clippy -- -D warnings` 경고 없음
