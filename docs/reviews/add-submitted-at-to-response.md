# 답변 응답에 submittedAt 필드 추가 리뷰

## 개요
답변 리스트 조회 API(API-020) 응답에 `submittedAt` 필드를 추가하여 프론트엔드에서 "12분 전"과 같은 상대 시간 표시가 가능하도록 합니다.

## 변경 사항

### 1. DTO 변경 (`dto.rs`)
- `ResponseListItem`에 `submitted_at: Option<chrono::NaiveDateTime>` 필드 추가
- JSON 직렬화 시 `submittedAt`(camelCase)로 출력

### 2. 서비스 변경 (`service.rs`)
- `member_retro` 테이블에서 해당 회고의 `submitted_at` 조회
- `member_id`를 키로 한 `member_submitted_at_map` 생성
- 각 응답의 작성자 `member_id`를 통해 `submitted_at` 매핑

### 3. 테스트 변경
- DTO 직렬화 단위 테스트에 `submitted_at` 필드 추가
- 통합 테스트 mock 데이터에 `submittedAt` 필드 추가
- camelCase 필드명 검증 테스트에 `submittedAt` 검증 추가

## 응답 형식

```json
{
  "responseId": 751,
  "userName": "아아",
  "content": "답변 내용",
  "likeCount": 0,
  "commentCount": 0,
  "submittedAt": "2026-02-27T18:00:00"
}
```

- `submittedAt`은 `member_retro.submitted_at` 값을 사용
- 제출 전 상태이거나 멤버 정보가 없는 경우 `null`

## 체크리스트
- [x] cargo test 통과
- [x] cargo clippy -- -D warnings 경고 없음
- [x] cargo fmt 적용
