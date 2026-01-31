# 핵심 개념: 회고 답변 임시 저장 (API-016)

## 부분 업데이트 (Partial Update)와 덮어쓰기

이 API는 `PUT` 메서드를 사용하지만, 리소스 전체를 교체하는 것이 아니라 **선택된 질문에 대한 답변만** 덮어쓰기 합니다.

- 회고 생성 시(`API-011`) 이미 5개의 질문(`Response`)이 생성되어 있습니다.
- 임시 저장 API는 새로운 레코드를 `INSERT` 하지 않고, 기존 레코드를 `UPDATE` 합니다.
- 클라이언트가 보낸 `drafts` 배열에 포함된 `questionNumber`에 해당하는 답변만 수정되며, 나머지 답변은 그대로 유지됩니다.

## ActiveModel을 이용한 업데이트

SeaORM에서 기존 데이터를 수정할 때는 `Model`을 `ActiveModel`로 변환하여 사용합니다.

```rust
let mut active: response::ActiveModel = response_model.clone().into();
active.content = Set(new_content);
active.updated_at = Set(now);
active.update(&txn).await?;
```

- `response_model.clone().into()`: DB에서 읽어온 불변 객체(`Model`)를 수정 가능한 객체(`ActiveModel`)로 변환합니다. 이때 모든 필드는 `Unchanged` 상태가 됩니다. `Unchanged` 상태의 필드는 `UPDATE` SQL에 포함되지 않으므로, 명시적으로 `Set()`으로 변경한 필드만 실제로 업데이트됩니다.
- `Set(val)`: 변경할 필드만 명시적으로 값을 설정합니다.
- `update()`: PK(`response_id`)를 기준으로 `UPDATE` 쿼리를 실행합니다.

> **DB 컬럼 매핑 주의**: `response` 엔티티의 Rust 필드명 `content`는 DB에서 `response`라는 컬럼에 매핑됩니다 (`#[sea_orm(column_name = "response")]`). SeaORM이 이를 자동으로 처리하므로 코드에서는 `active.content`로 접근할 수 있습니다.

## member_response 매핑을 통한 소유권 확인

회고 시스템 구조상 `Response` 테이블에는 `user_id` 컬럼이 직접 존재하지 않을 수 있습니다 (설계에 따라 다름). 이 프로젝트에서는 `member_response`라는 교차 테이블(junction table)을 통해 사용자와 응답을 연결합니다.

1.  **사용자 -> 응답 ID 목록**: `member_response` 테이블 조회
2.  **응답 ID 목록 + 회고 ID -> 응답 목록**: `response` 테이블 조회

이 과정을 통해 사용자가 **본인의 답변**만 수정할 수 있도록 강제합니다. 다른 사람의 답변 ID로 요청을 보내도 1번 단계에서 필터링되므로 안전합니다.

## Nullable 필드 처리 (`Option<String>`)

요청 DTO에서 `content` 필드는 `Option<String>` 타입입니다.

```rust
pub struct DraftItem {
    pub question_number: i32,
    pub content: Option<String>,
}
```

- JSON에서 `content` 필드가 없거나 `null`인 경우 Rust에서는 `None`이 됩니다.
- 서비스 로직에서는 이를 `unwrap_or_default()`를 사용하여 빈 문자열(`""`)로 변환해 저장합니다.
- 즉, 내용을 지우고 싶을 때 `null` 또는 `""`을 보내면 됩니다.

## 트랜잭션 내 반복 업데이트

여러 개의 답변을 수정할 때, **하나라도 실패하면 전체가 저장되지 않아야 합니다**.

```rust
let txn = state.db.begin().await?;
for draft in &req.drafts {
    // ... update ...
}
txn.commit().await?;
```

- 루프 안에서 `update`를 수행하지만, 실제 커밋은 루프가 끝난 뒤 `txn.commit()`에서 일어납니다.
- 중간에 에러가 발생하여 `?`로 함수를 탈출하면 `txn`이 drop되면서 롤백됩니다.
- 이를 통해 데이터 일관성을 보장합니다.

## 응답에서의 KST 변환

DB에는 UTC 시간을 저장하지만, API 응답의 `updated_at` 필드는 사용자에게 보여줄 한국 시간(KST)으로 변환합니다.

```rust
let kst_display = (now + chrono::Duration::hours(9))
    .format("%Y-%m-%d")
    .to_string();

Ok(DraftSaveResponse {
    retrospect_id,
    updated_at: kst_display,
})
```

- DB 저장: `Utc::now().naive_utc()` (UTC)
- 응답 표시: `UTC + 9시간` -> `"YYYY-MM-DD"` (KST)
- DB에 타임존 정보를 별도로 저장하지 않으므로, 코드 수준에서 UTC를 KST로 변환합니다.