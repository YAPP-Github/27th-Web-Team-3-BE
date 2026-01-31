# 핵심 개념: 회고 상세 조회 (API-012)

## Axum Handler 패턴: Extractor 기반 요청 파싱

Axum에서는 함수 시그니처의 매개변수 타입 자체가 요청 데이터를 추출하는 역할을 한다. 이를 **Extractor** 패턴이라 부른다.

**파일**: `codes/server/src/domain/retrospect/handler.rs:276-280`

```rust
pub async fn get_retrospect_detail(
    user: AuthUser,           // JWT 토큰에서 사용자 정보 추출
    State(state): State<AppState>,  // 앱 상태(DB 커넥션 등) 추출
    Path(retrospect_id): Path<i64>, // URL 경로에서 retrospectId 추출
) -> Result<Json<BaseResponse<RetrospectDetailResponse>>, AppError> {
```

- `AuthUser`: 커스텀 extractor로, `Authorization` 헤더에서 JWT를 파싱하고 인증 정보를 추출한다. 인증 실패 시 자동으로 401 에러를 반환한다.
- `State(state)`: Axum의 내장 extractor로, 라우터에 주입된 공유 상태(`AppState`)를 가져온다. 구조 분해 패턴(`State(state)`)으로 내부 값을 바로 꺼낸다.
- `Path(retrospect_id)`: URL 경로 변수를 `i64` 타입으로 자동 파싱한다.

Extractor는 선언 순서가 중요하다. `Body`를 소비하는 extractor(예: `Json`)는 마지막에 와야 한다.

## 응답 래핑 패턴: BaseResponse

모든 API 응답은 `BaseResponse<T>`로 일관된 구조를 유지한다.

**파일**: `codes/server/src/domain/retrospect/handler.rs:294-297`

```rust
Ok(Json(BaseResponse::success_with_message(
    result,
    "회고 상세 정보 조회를 성공했습니다.",
)))
```

응답 JSON 구조:
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회고 상세 정보 조회를 성공했습니다.",
  "result": { ... }
}
```

`BaseResponse::success_with_message()`는 제네릭 메서드로, `result` 필드에 어떤 `Serialize` 타입이든 담을 수 있다.

## SeaORM 엔티티 조회: find_by_id와 필터 체이닝

### 단건 조회 (find_by_id)

**파일**: `codes/server/src/domain/retrospect/service.rs:820-824`

```rust
let retrospect_model = retrospect::Entity::find_by_id(retrospect_id)
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?
    .ok_or_else(|| AppError::RetrospectNotFound("존재하지 않는 회고입니다.".to_string()))?;
```

- `find_by_id()`: Primary Key 기반 조회. `SELECT * FROM retrospect WHERE retrospect_id = ?` 와 동일하다.
- `.one()`: 단건 결과를 `Option<Model>`로 반환한다.
- `.map_err()`: SeaORM의 `DbErr`를 앱 에러로 변환한다.
- `.ok_or_else()`: `Option::None`을 에러로 변환한다.

두 개의 `?` 연산자가 연쇄적으로 사용되어, DB 에러와 데이터 미존재를 한 줄로 처리한다.

### 복합 조건 필터링

**파일**: `codes/server/src/domain/retrospect/service.rs:828-833`

```rust
let is_team_member = member_retro_room::Entity::find()
    .filter(member_retro_room::Column::MemberId.eq(user_id))
    .filter(member_retro_room::Column::RetrospectRoomId.eq(retrospect_room_id))
    .one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

- `Entity::find()`: 전체 조회 쿼리를 시작한다.
- `.filter()`: WHERE 조건을 추가한다. 여러 번 호출하면 AND 조건으로 결합된다.
- `Column::MemberId.eq(user_id)`: 타입 안전한 조건 표현식이다. SeaORM은 컴파일 타임에 칼럼 타입을 검증한다.

## 메모리 조인 패턴: 두 테이블의 별도 조회 후 HashMap 매핑

SeaORM에서 JOIN 대신, 두 테이블을 각각 조회한 후 메모리에서 결합하는 패턴이 사용된다.

**파일**: `codes/server/src/domain/retrospect/service.rs:842-890`

```rust
// 1단계: member_retro에서 참여자 목록 조회 (순서 보장)
let member_retros = member_retro::Entity::find()
    .filter(member_retro::Column::RetrospectId.eq(retrospect_id))
    .order_by_asc(member_retro::Column::MemberRetroId)
    .all(&state.db)
    .await?;

// 2단계: member_id 목록으로 member 테이블 IN 쿼리
let member_ids: Vec<i64> = member_retros.iter().map(|mr| mr.member_id).collect();
let members = member::Entity::find()
    .filter(member::Column::MemberId.is_in(member_ids))
    .all(&state.db)
    .await?;

// 3단계: HashMap으로 O(1) 룩업 구성
let member_map: HashMap<i64, String> = members.iter()
    .map(|m| (m.member_id, /* nickname */))
    .collect();

// 4단계: 원래 순서를 유지하면서 DTO 변환
let member_items: Vec<RetrospectMemberItem> = member_retros.iter()
    .filter_map(|mr| member_map.get(&mr.member_id).map(|n| /* DTO */))
    .collect();
```

이 패턴의 장점:
- **순서 보장**: `member_retro`의 정렬 순서를 유지하면서 `member` 데이터를 결합할 수 있다.
- **N+1 쿼리 방지**: 멤버마다 개별 조회하지 않고 IN 쿼리 1회로 해결한다.
- **유연성**: SeaORM의 관계 설정 없이도 테이블 간 데이터를 결합할 수 있다.

## HashSet을 이용한 중복 제거 및 순서 유지

**파일**: `codes/server/src/domain/retrospect/service.rs:903-913`

```rust
let mut seen_questions = HashSet::new();
let questions: Vec<RetrospectQuestionItem> = responses
    .iter()
    .filter(|r| seen_questions.insert(r.question.clone()))
    .take(5)
    .enumerate()
    .map(|(i, r)| RetrospectQuestionItem {
        index: (i + 1) as i32,
        content: r.question.clone(),
    })
    .collect();
```

`HashSet::insert()`는 값이 새로 삽입되면 `true`, 이미 존재하면 `false`를 반환한다. 이 특성을 `filter`의 조건으로 사용하여:
- 첫 번째 등장하는 질문만 통과시키고 (중복 제거)
- 원래 순서를 유지한다 (Iterator의 순서 보장)
- `.take(5)`로 최대 5개까지 제한한다

이는 별도의 질문 테이블이 없을 때, 응답 데이터에서 질문 목록을 추출하는 실용적인 패턴이다.

## PaginatorTrait::count()를 이용한 집계 쿼리

**파일**: `codes/server/src/domain/retrospect/service.rs:916-924`

```rust
let total_like_count = if response_ids.is_empty() {
    0
} else {
    response_like::Entity::find()
        .filter(response_like::Column::ResponseId.is_in(response_ids.clone()))
        .count(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))? as i64
};
```

- `count()`: SeaORM의 `PaginatorTrait`이 제공하는 메서드로, `SELECT COUNT(*) FROM ...` 쿼리를 실행한다.
- 반환 타입은 `u64`이므로 `as i64`로 캐스팅이 필요하다.
- 빈 목록 방어: `response_ids`가 비어있으면 DB 쿼리를 하지 않고 0을 반환한다. 빈 `IN()` 절은 DB에 따라 에러를 발생시킬 수 있기 때문이다.

## 접근 권한 분리 패턴: 403 vs 404

이 API에서는 403(접근 권한 없음)과 404(회고 없음)를 명확하게 분리한다.

**파일**: `codes/server/src/domain/retrospect/service.rs:819-839`

```rust
// 404: 회고가 존재하지 않는 경우
.ok_or_else(|| AppError::RetrospectNotFound("존재하지 않는 회고입니다.".to_string()))?;

// 403: 회고는 존재하지만 접근 권한이 없는 경우
if is_team_member.is_none() {
    return Err(AppError::TeamAccessDenied(
        "해당 회고에 접근 권한이 없습니다.".to_string(),
    ));
}
```

반면, 다른 API(예: `find_retrospect_for_member` 헬퍼, 라인 323-352)에서는 보안상 404로 통합하는 패턴을 사용한다:

```rust
// 회고 미존재와 권한 없음을 모두 404로 반환 (보안상 정보 노출 방지)
.ok_or_else(|| AppError::RetrospectNotFound(
    "존재하지 않는 회고이거나 접근 권한이 없습니다.".to_string(),
))?;
```

상세 조회 API는 프론트엔드에서 에러 유형별로 다른 UI를 보여줘야 하므로 분리된 에러 코드를 사용한다.

## Option 처리: filter와 unwrap_or_else 체이닝

**파일**: `codes/server/src/domain/retrospect/service.rs:864-868`

```rust
let nickname = m
    .nickname
    .clone()
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| "Unknown".to_string());
```

`Option<String>`에 대한 안전한 처리 체인:
1. `.clone()`: `Option<String>`을 복제한다 (소유권 이동 없이).
2. `.filter(|s| !s.is_empty())`: 빈 문자열이면 `None`으로 변환한다.
3. `.unwrap_or_else(|| "Unknown".to_string())`: `None`이면 기본값 "Unknown"을 사용한다.

이 패턴은 `unwrap()`이나 `expect()`를 사용하지 않으면서도 간결하게 기본값 처리를 수행한다.

## chrono::NaiveDateTime과 날짜 포맷팅

DB에서 가져온 `start_time`은 SeaORM의 `DateTime` 타입이며, 이는 `chrono::NaiveDateTime`의 타입 별칭이다. `NaiveDateTime`은 타임존 정보가 없는 날짜+시간 값을 나타낸다.

**파일**: `codes/server/src/domain/retrospect/entity/retrospect.rs:83`

```rust
// SeaORM 엔티티에서의 타입 선언
pub start_time: DateTime,  // DateTime = chrono::NaiveDateTime
```

**파일**: `codes/server/src/domain/retrospect/service.rs:937-938`

```rust
// NaiveDateTime을 strftime 포맷 문자열로 변환
let start_time = retrospect_model.start_time.format("%Y-%m-%d").to_string();
```

### chrono 주요 타입 비교

| 타입 | 설명 | 타임존 |
|------|------|--------|
| `NaiveDate` | 날짜만 (예: 2026-01-24) | 없음 |
| `NaiveTime` | 시간만 (예: 14:30:00) | 없음 |
| `NaiveDateTime` | 날짜+시간 (예: 2026-01-24T14:30:00) | 없음 |
| `DateTime<Utc>` | UTC 기준 날짜+시간 | UTC |

### 이 프로젝트의 타임존 전략

이 프로젝트에서는 `NaiveDateTime`을 사용하되, **KST(한국 시간, UTC+9) 값을 직접 저장**하는 방식을 채택한다:

```rust
// 회고 생성 시: 입력받은 KST 날짜/시간을 NaiveDateTime으로 조합하여 저장 (service.rs:120)
let start_time = NaiveDateTime::new(retrospect_date, retrospect_time);

// 현재 시간 비교 시: UTC 기준에 9시간을 더해 KST로 변환 (service.rs:261)
let now_kst = Utc::now().naive_utc() + chrono::Duration::hours(9);

// 상세 조회 시: 저장된 KST 값을 포맷만 하면 됨 (service.rs:938)
let start_time = retrospect_model.start_time.format("%Y-%m-%d").to_string();
```

### format() 메서드의 strftime 지시자

| 지시자 | 의미 | 예시 |
|--------|------|------|
| `%Y` | 4자리 연도 | 2026 |
| `%m` | 2자리 월 (01~12) | 01 |
| `%d` | 2자리 일 (01~31) | 24 |
| `%H` | 24시간제 시 (00~23) | 14 |
| `%M` | 분 (00~59) | 30 |

이 API에서는 `"%Y-%m-%d"` 포맷으로 날짜 부분만 추출한다.

---

## filter_map을 이용한 안전한 변환 및 경고 로깅

**파일**: `codes/server/src/domain/retrospect/service.rs:874-890`

```rust
let member_items: Vec<RetrospectMemberItem> = member_retros
    .iter()
    .filter_map(|mr| {
        let name = member_map.get(&mr.member_id);
        if name.is_none() {
            warn!(
                member_id = mr.member_id,
                retrospect_id = retrospect_id,
                "member_retro에 등록되어 있으나 member 테이블에 존재하지 않는 멤버"
            );
        }
        name.map(|n| RetrospectMemberItem {
            member_id: mr.member_id,
            user_name: n.clone(),
        })
    })
    .collect();
```

`filter_map`은 `filter`와 `map`을 결합한 Iterator 어댑터이다:
- `Some(value)` 반환 시: 결과에 포함
- `None` 반환 시: 결과에서 제외

여기서는 `member_map`에 없는 멤버를 안전하게 건너뛰되, `warn!` 매크로로 데이터 정합성 문제를 로그에 기록한다. 서비스가 에러를 내지 않으면서도 이상 상황을 추적할 수 있다.
