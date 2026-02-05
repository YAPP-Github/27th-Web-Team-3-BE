# 동작 흐름: 회고 생성 (API-011)

## 전체 흐름 요약

```
클라이언트 요청
  -> [Axum] AuthUser extractor (JWT 인증)
  -> [Handler] create_retrospect (DTO 검증: req.validate())
  -> [Service] create_retrospect (비즈니스 로직)
    -> validate_reference_urls() (URL 중복/형식/길이 검증)
    -> validate_and_parse_date() (chrono::NaiveDate 파싱 + 오늘 이후 확인)
    -> validate_and_parse_time() (chrono::NaiveTime 파싱)
    -> validate_future_datetime() (NaiveDateTime 결합, KST 기준 UTC+9 미래 검증)
    -> DB: 팀 존재 확인
    -> DB: 팀 멤버십 확인
    -> 트랜잭션 시작
      -> RetroRoom 생성 (INVITATION_BASE_URL + UUID v4로 초대 URL 생성)
      -> Retrospect 생성 (제목, 방식, NaiveDateTime으로 start_time 저장)
      -> Response × 5 생성 (기본 질문 5개, 빈 content)
      -> RetroReference × N 생성 (참고 URL 저장)
    -> 트랜잭션 커밋
  -> [Handler] BaseResponse 래핑 후 JSON 응답
```

## 단계별 상세

### 1단계: JWT 인증 (AuthUser Extractor)

**파일**: `src/utils/auth.rs`

Axum의 `FromRequestParts` trait을 구현한 `AuthUser`가 요청에서 자동으로 JWT를 추출 및 검증합니다.

```rust
pub struct AuthUser(pub Claims);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // Authorization 헤더에서 "Bearer {token}" 추출
        // decode_access_token으로 JWT 검증
    }
}
```

- `Authorization` 헤더가 없으면 `AppError::Unauthorized` 반환
- `Bearer ` 접두사가 없으면 `AppError::Unauthorized` 반환
- 토큰이 유효하지 않으면 `AppError::Unauthorized` 반환

### 2단계: 핸들러 - 입력 검증 및 서비스 호출

**파일**: `src/domain/retrospect/handler.rs:45-63`

```rust
pub async fn create_retrospect(
    user: AuthUser,                          // 1단계에서 자동 추출
    State(state): State<AppState>,           // 앱 상태 (DB 커넥션 등)
    Json(req): Json<CreateRetrospectRequest>,// JSON 바디 자동 역직렬화
) -> Result<Json<BaseResponse<CreateRetrospectResponse>>, AppError> {
    req.validate()?;           // validator 크레이트로 DTO 검증
    let user_id = user.user_id()?;
    let result = RetrospectService::create_retrospect(state, user_id, req).await?;
    Ok(Json(BaseResponse::success_with_message(result, "...")))
}
```

핸들러의 역할:
- `Json(req)`: Content-Type: application/json 바디를 `CreateRetrospectRequest`로 자동 역직렬화
- `req.validate()?`: validator 크레이트의 `Validate` trait으로 DTO 검증
- `user.user_id()?`: Claims의 `sub` 필드를 `i64`로 파싱
- 비즈니스 로직은 Service에 위임

### 3단계: 서비스 - URL 검증

**파일**: `src/domain/retrospect/service.rs:187-226`

```rust
fn validate_reference_urls(urls: &[String]) -> Result<(), AppError> {
    // 중복 검증: HashSet으로 O(n) 중복 감지
    let unique_urls: HashSet<_> = urls.iter().collect();
    if unique_urls.len() != urls.len() { ... }

    for url in urls {
        // 길이 검증
        if url.len() > REFERENCE_URL_MAX_LENGTH { ... }

        // 스키마 검증 (if let 패턴 매칭)
        let without_scheme = if let Some(stripped) = url.strip_prefix("https://") {
            stripped
        } else if let Some(stripped) = url.strip_prefix("http://") {
            stripped
        } else {
            return Err(AppError::RetroUrlInvalid("유효하지 않은 URL 형식입니다.".to_string()));
        };

        // 호스트 존재 검증
        if without_scheme.is_empty() || !without_scheme.contains('.') { ... }
    }
}
```

- `HashSet`으로 중복 URL 감지
- `if let Some(stripped)` 패턴으로 `http://` 또는 `https://` 스키마 검증
- 스키마 제거 후 `.` 포함 여부로 호스트 존재 확인

### 4단계: 서비스 - 날짜/시간 검증 (chrono 크레이트)

날짜와 시간을 각각 `chrono` 타입으로 파싱한 뒤, 결합하여 KST 기준 미래 시점인지 검증합니다.

**파일**: `src/domain/retrospect/service.rs:228-270`

```rust
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};

/// 날짜 형식 및 미래 날짜 검증
fn validate_and_parse_date(date_str: &str) -> Result<NaiveDate, AppError> {
    // "2026-01-20" → NaiveDate 파싱
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("날짜 형식이 올바르지 않습니다. (YYYY-MM-DD 형식 필요)".to_string()))?;

    // 오늘 이후 날짜 검증 (오늘 포함)
    let today = Utc::now().date_naive();
    if date < today {
        return Err(AppError::BadRequest("회고 날짜는 오늘 이후만 허용됩니다.".to_string()));
    }
    Ok(date)
}

/// 시간 형식 검증
fn validate_and_parse_time(time_str: &str) -> Result<NaiveTime, AppError> {
    // "10:00" → NaiveTime 파싱
    NaiveTime::parse_from_str(time_str, "%H:%M")
        .map_err(|_| AppError::BadRequest("시간 형식이 올바르지 않습니다. (HH:mm 형식 필요)".to_string()))
}

/// 미래 날짜/시간 검증 (한국 시간 기준, UTC+9)
fn validate_future_datetime(date: NaiveDate, time: NaiveTime) -> Result<(), AppError> {
    let input_datetime = NaiveDateTime::new(date, time);

    // 한국 시간 기준 현재 시각 (UTC + 9시간)
    let now_kst = Utc::now().naive_utc() + chrono::Duration::hours(9);

    if input_datetime <= now_kst {
        return Err(AppError::BadRequest("회고 날짜와 시간은 현재보다 미래여야 합니다.".to_string()));
    }
    Ok(())
}
```

- `NaiveDate`: 타임존 없는 날짜 (`2026-01-20`)
- `NaiveTime`: 타임존 없는 시간 (`10:00`)
- `NaiveDateTime`: 날짜+시간 결합 (`2026-01-20T10:00:00`)
- `Utc::now().date_naive()`: UTC 기준 오늘 날짜를 `NaiveDate`로 변환
- `Utc::now().naive_utc() + Duration::hours(9)`: UTC에 9시간을 더해 KST 시각 계산
- **주의**: `chrono-tz` 크레이트 없이 수동 오프셋(+9)으로 KST를 처리하는 경량 방식

### 5단계: 서비스 - 팀 존재 여부 & 멤버십 확인

```rust
// 팀 존재 확인
let team_exists = team::Entity::find_by_id(team_id)
    .one(&state.db).await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

// 팀 멤버십 확인
let is_member = member_team::Entity::find()
    .filter(member_team::Column::MemberId.eq(user_id))
    .filter(member_team::Column::TeamId.eq(team_id))
    .one(&state.db).await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

- 팀이 없으면 404 에러 (`TEAM4041`)
- 멤버가 아니면 403 에러 (`TEAM4031`)

### 6단계: 서비스 - 트랜잭션으로 데이터 생성

**파일**: `src/domain/retrospect/service.rs:87-183`

```rust
// 트랜잭션 시작
let txn = state.db.begin().await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

// 현재 시각 (created_at, updated_at 용)
let now = Utc::now().naive_utc();

// 1. RetroRoom 생성 (초대 URL 포함)
let base_url = std::env::var("INVITATION_BASE_URL")
    .unwrap_or_else(|_| "https://retro.example.com".to_string());
let invitation_url = format!(
    "{}/room/{}",
    base_url.trim_end_matches('/'),
    uuid::Uuid::new_v4()
);
let retro_room_model = retro_room::ActiveModel {
    title: Set(req.project_name.clone()),
    invitation_url: Set(invitation_url),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
};
let retro_room_result = retro_room_model.insert(&txn).await?;

// 2. Retrospect 생성 (날짜+시간을 NaiveDateTime으로 결합)
let start_time = NaiveDateTime::new(retrospect_date, retrospect_time);
let retrospect_model = retrospect::ActiveModel {
    title: Set(req.project_name.clone()),
    team_insight: Set(None),
    retrospect_method: Set(req.retrospect_method.clone()),
    created_at: Set(now),
    updated_at: Set(now),
    start_time: Set(start_time),
    retrospect_room_id: Set(retro_room_result.retrospect_room_id),
    team_id: Set(req.team_id),
    ..Default::default()
};
let retrospect_result = retrospect_model.insert(&txn).await?;

// 3. Response × 5 생성 (회고 방식별 기본 질문)
for question in req.retrospect_method.default_questions() {
    let response_model = response::ActiveModel {
        question: Set(question.to_string()),
        content: Set(String::new()),
        created_at: Set(now),
        updated_at: Set(now),
        retrospect_id: Set(retrospect_result.retrospect_id),
        ..Default::default()
    };
    response_model.insert(&txn).await?;
}

// 4. RetroReference × N 생성 (참고 URL)
for url in &req.reference_urls {
    let reference_model = retro_reference::ActiveModel {
        title: Set(url.clone()),
        url: Set(url.clone()),
        retrospect_id: Set(retrospect_result.retrospect_id),
        ..Default::default()
    };
    reference_model.insert(&txn).await?;
}

// 트랜잭션 커밋
txn.commit().await
    .map_err(|e| AppError::InternalError(e.to_string()))?;
```

- 4개 테이블에 데이터를 하나의 트랜잭션으로 삽입
- `Utc::now().naive_utc()`: 현재 UTC 시각을 `NaiveDateTime`으로 변환하여 `created_at`/`updated_at`에 사용
- `NaiveDateTime::new(date, time)`: 별도 파싱한 날짜와 시간을 결합하여 `start_time` 필드에 저장
- `INVITATION_BASE_URL` 환경변수와 UUID v4를 조합하여 초대 URL 생성
- 중간에 에러 발생 시 `txn` drop으로 자동 롤백 (RAII 패턴)

### 7단계: 응답 래핑

```rust
Ok(Json(BaseResponse::success_with_message(
    result,
    "회고가 성공적으로 생성되었습니다.",
)))
```

## 에러 흐름

| 단계 | 조건 | 에러 코드 | HTTP 상태 |
|------|------|-----------|-----------|
| 1 (인증) | 토큰 없음/만료/잘못된 형식 | AUTH4001 | 401 |
| 2 (핸들러) | DTO 검증 실패 (project_name 등) | RETRO4001~4006 | 400 |
| 3 (서비스) | URL 중복/형식/길이 오류 | RETRO4006 | 400 |
| 4 (서비스) | 날짜/시간 형식 오류, 과거 시점 | RETRO4003~4005 | 400 |
| 5 (서비스) | 팀이 DB에 없음 | TEAM4041 | 404 |
| 5 (서비스) | 사용자가 팀 멤버가 아님 | TEAM4031 | 403 |
| 6 (서비스) | DB 쿼리/트랜잭션 실패 | COMMON500 | 500 |
