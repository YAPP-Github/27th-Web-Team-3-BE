# 학습 키워드: 회고 상세 조회 (API-012)

## 1. Entity::find_by_id

Primary Key 기반 단건 조회 메서드. SeaORM이 엔티티 정의에서 PK를 자동 인식하여 WHERE 절을 생성한다.

```rust
retrospect::Entity::find_by_id(retrospect_id)
    .one(&state.db)
    .await
```

- **코드 위치**: `service.rs:820`
- **반환 타입**: `Result<Option<Model>, DbErr>`
- **SQL 등가**: `SELECT * FROM retrospect WHERE retrospect_id = $1`

---

## 2. filter 체이닝 (AND 조건)

SeaORM에서 `.filter()`를 여러 번 호출하면 AND 조건으로 결합된다.

```rust
member_retro_room::Entity::find()
    .filter(member_retro_room::Column::MemberId.eq(user_id))
    .filter(member_retro_room::Column::RetrospectRoomId.eq(retrospect_room_id))
    .one(&state.db)
```

- **코드 위치**: `service.rs:828-833`
- **SQL 등가**: `SELECT * FROM member_retro_room WHERE member_id = $1 AND retrospect_room_id = $2 LIMIT 1`

---

## 3. Column::is_in (IN 쿼리)

여러 값을 한번에 조회할 때 사용하는 SeaORM 조건 표현식. N+1 쿼리 문제를 방지한다.

```rust
member::Entity::find()
    .filter(member::Column::MemberId.is_in(member_ids))
    .all(&state.db)
```

- **코드 위치**: `service.rs:854-856`
- **SQL 등가**: `SELECT * FROM member WHERE member_id IN ($1, $2, ...)`
- **주의**: 빈 벡터를 전달하면 `IN ()` 구문이 되어 DB 에러가 발생할 수 있으므로, 호출 전에 빈 목록 검사가 필요하다 (라인 851-853).

---

## 4. PaginatorTrait::count

`SELECT COUNT(*)` 쿼리를 실행하여 레코드 수를 반환하는 SeaORM 메서드.

```rust
response_like::Entity::find()
    .filter(response_like::Column::ResponseId.is_in(response_ids.clone()))
    .count(&state.db)
    .await
```

- **코드 위치**: `service.rs:919-922` (좋아요), `service.rs:930-933` (댓글)
- **반환 타입**: `Result<u64, DbErr>` -- `i64`로 캐스팅 필요 (`as i64`)
- **import**: `sea_orm::PaginatorTrait` (`service.rs:8`)

---

## 5. HashMap을 이용한 메모리 조인

두 테이블을 별도로 조회한 후, HashMap으로 O(1) 룩업을 구성하여 메모리에서 결합하는 패턴.

```rust
let member_map: HashMap<i64, String> = members.iter()
    .map(|m| (m.member_id, nickname))
    .collect();
```

- **코드 위치**: `service.rs:861-871`
- **장점**: 순서 보장, N+1 방지, 관계 설정 불필요
- **관련 import**: `std::collections::HashMap` (`service.rs:1`)

---

## 6. HashSet::insert를 이용한 중복 제거

`HashSet::insert()`가 반환하는 `bool` 값(새 삽입이면 `true`)을 `filter` 조건으로 활용하여, Iterator 순서를 유지하면서 중복을 제거한다.

```rust
let mut seen_questions = HashSet::new();
let questions: Vec<RetrospectQuestionItem> = responses
    .iter()
    .filter(|r| seen_questions.insert(r.question.clone()))
    .take(5)
    .enumerate()
    .map(|(i, r)| RetrospectQuestionItem { index: (i + 1) as i32, content: r.question.clone() })
    .collect();
```

- **코드 위치**: `service.rs:903-913`
- **관련 import**: `std::collections::HashSet` (`service.rs:1`)
- **활용**: 별도 질문 테이블 없이, 응답 테이블에서 고유 질문을 추출

---

## 7. filter_map (안전한 변환 + 필터링)

`Option`을 반환하면서 `Some`인 경우만 결과에 포함시키는 Iterator 어댑터. `filter`와 `map`을 합친 것이다.

```rust
member_retros.iter()
    .filter_map(|mr| {
        member_map.get(&mr.member_id)
            .map(|n| RetrospectMemberItem { member_id: mr.member_id, user_name: n.clone() })
    })
    .collect();
```

- **코드 위치**: `service.rs:874-890`
- **용도**: HashMap에 없는 멤버를 안전하게 건너뛰면서, 데이터 정합성 문제를 `warn!` 로그로 기록

---

## 8. Option::filter + unwrap_or_else 체인

`Option<String>`에서 빈 문자열을 `None`으로 변환한 뒤, 기본값을 제공하는 안전한 패턴.

```rust
m.nickname
    .clone()
    .filter(|s| !s.is_empty())      // 빈 문자열이면 None으로
    .unwrap_or_else(|| "Unknown".to_string())  // None이면 기본값
```

- **코드 위치**: `service.rs:864-868`
- **장점**: `unwrap()` / `expect()` 없이 안전하게 기본값 처리

---

## 9. 보안상 404 통합 패턴 vs 403/404 분리 패턴

### 통합 패턴 (find_retrospect_for_member 헬퍼)

존재하지 않는 리소스와 접근 권한 없음을 모두 404로 반환하여, 리소스 존재 여부를 외부에 노출하지 않는다.

```rust
.ok_or_else(|| AppError::RetrospectNotFound(
    "존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string(),
))
```

- **코드 위치**: `service.rs:332-336`, `service.rs:346-348`
- **사용 API**: API-014(참석자 등록), API-018(참고자료 조회) 등

### 분리 패턴 (API-012 상세 조회)

404와 403을 분리하여 프론트엔드에서 에러 유형별 UI 처리가 가능하다.

```rust
// 404
.ok_or_else(|| AppError::RetrospectNotFound("존재하지 않는 회고입니다.".to_string()))?;
// 403
return Err(AppError::TeamAccessDenied("해당 회고에 접근 권한이 없습니다.".to_string()));
```

- **코드 위치**: `service.rs:824`, `service.rs:836-838`
- **사용 시점**: 에러 유형별 사용자 안내가 필요한 화면 진입 API

---

## 10. AuthUser Extractor (JWT 인증)

Axum의 커스텀 extractor로, 요청 헤더에서 JWT 토큰을 자동 추출하고 검증한다.

```rust
pub async fn get_retrospect_detail(
    user: AuthUser,   // 자동으로 Authorization 헤더에서 JWT 파싱
    ...
) -> Result<..., AppError> {
    let user_id = user.user_id()?;  // JWT 클레임에서 user_id 추출
}
```

- **코드 위치**: `handler.rs:277`, `handler.rs:289`
- **인증 실패 시**: 401 Unauthorized 자동 반환 (핸들러 함수 진입 전 처리)

---

## 11. order_by_asc (정렬)

SeaORM에서 결과 정렬을 위한 메서드. `QueryOrder` 트레이트가 제공한다.

```rust
member_retro::Entity::find()
    .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
    .order_by_asc(member_retro::Column::MemberRetroId)
    .all(&state.db)
```

- **코드 위치**: `service.rs:842-847`
- **import**: `sea_orm::QueryOrder` (`service.rs:8`)
- **용도**: 참석 등록일(PK) 기준 오름차순 정렬로 가입 순서를 보장

---

## 12. serde rename_all = "camelCase"

Rust의 `snake_case` 필드명을 JSON의 `camelCase`로 자동 변환하는 serde 어트리뷰트.

```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetrospectDetailResponse {
    pub team_id: i64,          // JSON: "teamId"
    pub total_like_count: i64, // JSON: "totalLikeCount"
    ...
}
```

- **코드 위치**: `dto.rs:374-393`
- **프로젝트 규칙**: 모든 DTO에 필수 적용 (CLAUDE.md 참조)

---

## 13. chrono::NaiveDateTime과 날짜 포맷팅

SeaORM의 `DateTime` 타입은 `chrono::NaiveDateTime`의 별칭이다. `NaiveDateTime`은 타임존 정보가 없는 날짜+시간 값이며, `.format()` 메서드로 strftime 스타일의 문자열로 변환한다.

```rust
// DB에서 조회한 start_time(NaiveDateTime)을 "YYYY-MM-DD" 문자열로 변환
let start_time = retrospect_model.start_time.format("%Y-%m-%d").to_string();
```

- **코드 위치**: `service.rs:938` (상세 조회), `dto.rs:204-205` (팀 목록에서도 동일 패턴)
- **엔티티 타입 선언**: `entity/retrospect.rs:83` (`pub start_time: DateTime`)
- **import**: `chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc}` (`service.rs:3`)
- **타임존 전략**: KST 값을 `NaiveDateTime`에 직접 저장. 현재 시간 비교 시에는 `Utc::now().naive_utc() + chrono::Duration::hours(9)` 로 KST를 구한다 (`service.rs:261`).
- **주요 포맷 지시자**: `%Y`(4자리 연도), `%m`(2자리 월), `%d`(2자리 일), `%H`(24시간제 시), `%M`(분)

---

## 14. map_err + ? 연산자 (에러 변환 체인)

SeaORM의 `DbErr`를 앱 에러(`AppError`)로 변환하면서 `?` 연산자로 에러를 전파하는 패턴.

```rust
.map_err(|e| AppError::InternalError(e.to_string()))?
```

- **코드 위치**: `service.rs:822-823`, `service.rs:832-833` 등 (서비스 전반에서 반복 사용)
- **역할**: DB 에러를 500 Internal Error로 변환하여 상위 레이어에 전파
