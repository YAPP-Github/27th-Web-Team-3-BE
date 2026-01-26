# API-010 RetroRoom Retrospects List Implementation Review

## 개요
- **API**: `GET /api/v1/retro-rooms/{retroRoomId}/retrospects`
- **기능**: 레트로룸 내 회고 목록 조회
- **담당자**: Claude Code
- **작성일**: 2026-01-26
- **상태**: 구현 완료

## API 스펙 요약

### Request
```
GET /api/v1/retro-rooms/{retroRoomId}/retrospects
Authorization: Bearer {accessToken}
```

### Response (200 OK)
```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "성공입니다.",
  "result": [
    {
      "retrospectId": 100,
      "projectName": "프로젝트 회고",
      "retrospectMethod": "KPT",
      "retrospectDate": "2026-01-20",
      "retrospectTime": "10:00"
    },
    {
      "retrospectId": 101,
      "projectName": "스프린트 회고",
      "retrospectMethod": "KPT",
      "retrospectDate": "2026-01-24",
      "retrospectTime": "16:00"
    }
  ]
}
```

## 구현 상세

### 1. DTO 설계 (`domain/retrospect/dto.rs`)

```rust
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetrospectListItem {
    pub retrospect_id: i64,
    pub project_name: String,
    pub retrospect_method: String,
    pub retrospect_date: String,
    pub retrospect_time: String,
}
```

### 2. 비즈니스 로직 (`domain/retrospect/service.rs`)

```rust
pub async fn list_retrospects(
    state: AppState,
    member_id: i64,
    retro_room_id: i64,
) -> Result<Vec<RetrospectListItem>, AppError> {
    // 1. 룸 존재 확인
    // 2. 멤버 권한 확인 (해당 룸의 멤버인지)
    // 3. 회고 목록 조회 (start_time 기준 최신순 정렬)
}
```

**처리 순서**:
1. `retro_room_id`로 룸 존재 확인
2. `member_retro_room`에서 멤버 여부 확인
3. `retrospects` 테이블에서 해당 룸의 회고 조회
4. `start_time` 기준 내림차순(최신순) 정렬

### 3. 핸들러 (`domain/retrospect/handler.rs`)

```rust
#[utoipa::path(
    get,
    path = "/api/v1/retro-rooms/{retro_room_id}/retrospects",
    params(
        ("retro_room_id" = i64, Path, description = "레트로룸 ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "회고 목록 조회 성공", body = SuccessRetrospectListResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 403, description = "권한 없음", body = ErrorResponse),
        (status = 404, description = "룸 없음", body = ErrorResponse)
    ),
    tag = "RetroRoom"
)]
pub async fn list_retrospects(...)
```

## 에러 처리

| 에러 코드 | HTTP 상태 | 설명 |
|-----------|-----------|------|
| `AUTH4001` | 401 | 인증 정보가 유효하지 않음 |
| `TEAM4031` | 403 | 접근 권한 없음 (멤버가 아님) |
| `RETRO4041` | 404 | 존재하지 않는 레트로룸 |
| `COMMON500` | 500 | 서버 내부 에러 |

## 응답 필드 매핑

| Response 필드 | Entity 필드 | 변환 |
|---------------|-------------|------|
| `retrospectId` | `retrospect_id` | 그대로 |
| `projectName` | `title` | 그대로 |
| `retrospectMethod` | `retro_category` | Enum → String (대문자) |
| `retrospectDate` | `start_time` | `%Y-%m-%d` 포맷 |
| `retrospectTime` | `start_time` | `%H:%M` 포맷 |

## retrospectMethod Enum

| 값 | 설명 |
|----|------|
| KPT | Keep-Problem-Try 방식 |
| FOUR_L | 4L (Liked, Learned, Lacked, Longed For) 방식 |
| FIVE_F | 5F (Facts, Feelings, Findings, Future, Feedback) 방식 |
| PMI | Plus-Minus-Interesting 방식 |
| FREE | 자유 형식 |

> **현재 구현**: `retro_category` 필드가 `KPT`만 지원. 추후 확장 필요.

## 코드 리뷰 체크리스트

- [x] 멤버 권한 검증이 작동하는가?
- [x] 정렬 순서가 올바른가? (start_time DESC)
- [x] 빈 배열 응답이 처리되는가?
- [x] 날짜/시간 포맷이 올바른가?
- [x] Swagger 문서가 생성되는가?
- [x] 코드가 Rust 컨벤션을 따르는가?
