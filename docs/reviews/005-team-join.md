# API-005 Retro Room Join Implementation Review

## 개요
- **API**: `POST /api/v1/retro-rooms/join`
- **기능**: 초대 URL을 통한 레트로룸(Retro Room) 합류
- **담당자**: Claude Code
- **작성일**: 2026-01-26
- **상태**: 구현 완료

## API 스펙 요약

### Request
```json
{
  "inviteUrl": "https://service.com/invite/INV-A1B2-C3D4"
}
```

### Response (200 OK)
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": {
    "retroRoomId": 789,
    "title": "코드 마스터즈",
    "joinedAt": "2026-01-24T15:45:00"
  }
}
```

## 구현 상세

### 1. DTO 설계 (`domain/retrospect/dto.rs`)

**Request**:
```rust
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JoinRetroRoomRequest {
    #[validate(url(message = "유효한 URL 형식이 아닙니다."))]
    pub invite_url: String,
}
```

**Response**:
```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JoinRetroRoomResponse {
    pub retro_room_id: i64,
    pub title: String,
    pub joined_at: String,
}
```

### 2. 비즈니스 로직 (`domain/retrospect/service.rs`)

**초대 코드 추출 로직**:
- Path segment 형식: `https://service.com/invite/INV-A1B2-C3D4`
- Query parameter 형식: `https://service.com/join?code=INV-A1B2-C3D4`

```rust
fn extract_invite_code(invite_url: &str) -> Result<String, AppError> {
    // 1. 쿼리 파라미터 형식 확인 (?code=...)
    if let Some(query_start) = invite_url.find('?') {
        let query_string = &invite_url[query_start + 1..];
        for param in query_string.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "code" && value.starts_with("INV-") {
                    return Ok(value.to_string());
                }
            }
        }
    }

    // 2. Path segment 형식 확인
    let path = invite_url.split('?').next().unwrap_or(invite_url);
    if let Some(last_segment) = path.split('/').next_back() {
        if last_segment.starts_with("INV-") {
            return Ok(last_segment.to_string());
        }
    }

    Err(AppError::InvalidInviteLink("유효하지 않은 초대 링크입니다.".into()))
}
```

**합류 프로세스**:
1. URL에서 초대 코드 추출
2. 초대 코드로 `retro_room` 테이블 조회
3. 초대 코드 만료 체크 (생성일 기준 7일)
4. 중복 가입 체크 (`member_retro_room` 테이블)
5. `member_retro_room` 테이블에 `MEMBER` 권한으로 추가

### 3. 에러 처리 (`utils/error.rs`)

| 에러 코드 | HTTP 상태 | 설명 |
|-----------|-----------|------|
| `RETRO4002` | 400 | 유효하지 않은 초대 링크 |
| `RETRO4003` | 400 | 만료된 초대 링크 (7일 초과) |
| `RETRO4041` | 404 | 존재하지 않는 회고 룸 |
| `RETRO4092` | 409 | 이미 해당 룸의 멤버 |

### 4. 핸들러 (`domain/retrospect/handler.rs`)

```rust
#[utoipa::path(
    post,
    path = "/api/v1/retro-rooms/join",
    request_body = JoinRetroRoomRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "회고 룸 참여 성공", body = SuccessJoinRetroRoomResponse),
        (status = 400, description = "잘못된 초대 링크 또는 만료됨", body = ErrorResponse),
        (status = 404, description = "존재하지 않는 룸", body = ErrorResponse),
        (status = 409, description = "이미 참여 중", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn join_retro_room(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<JoinRetroRoomRequest>,
) -> Result<Json<BaseResponse<JoinRetroRoomResponse>>, AppError>
```

## 테스트

### 단위 테스트 (service.rs)
```
running 6 tests
test should_extract_invite_code_from_path_segment ... ok
test should_extract_invite_code_from_query_parameter ... ok
test should_extract_invite_code_from_query_with_multiple_params ... ok
test should_return_error_for_invalid_url ... ok
test should_return_error_for_empty_code ... ok
test should_generate_valid_invite_code ... ok
```

### 테스트 커버리지
- [x] Path segment에서 초대 코드 추출
- [x] Query parameter에서 초대 코드 추출
- [x] 다중 query parameter에서 초대 코드 추출
- [x] 유효하지 않은 URL 에러 처리
- [x] 빈 코드 에러 처리
- [x] 초대 코드 생성 검증

## 코드 리뷰 체크리스트

- [x] TDD 원칙을 따라 테스트 코드가 작성되었는가?
- [x] 모든 테스트가 통과하는가?
- [x] API 문서가 `docs/reviews/` 디렉토리에 작성되었는가?
- [x] 공통 유틸리티를 재사용했는가? (`BaseResponse`, `AppError`)
- [x] 에러 처리가 적절하게 되어 있는가?
- [x] 코드가 Rust 컨벤션을 따르는가? (`cargo fmt`, `cargo clippy`)
- [x] 불필요한 의존성이 추가되지 않았는가?

## 품질 검사 결과

```bash
$ cargo fmt    # 통과
$ cargo clippy -- -D warnings  # 통과
$ cargo test   # 8 passed
```

## 변경 파일 목록

| 파일 | 변경 유형 | 설명 |
|------|-----------|------|
| `domain/retrospect/service.rs` | 수정 | `extract_invite_code` 함수 추가, 테스트 추가 |
| `domain/auth/handler.rs` | 수정 | unused import 경고 수정 |
| `config/app_config.rs` | 수정 | dead_code 경고 수정 |
| `docs/reviews/005-team-join.md` | 수정 | 구현 리뷰 문서 업데이트 |
