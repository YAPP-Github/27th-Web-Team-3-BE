# 학습 키워드: 회고 답변 임시 저장 (API-016)

## 1. `PUT` vs `PATCH` (RESTful 설계)

- **PUT**: 리소스의 전체 교체. 보내지 않은 필드는 null/삭제 처리되는 것이 원칙.
- **PATCH**: 리소스의 부분 수정. 보낸 필드만 변경.
- **이 API의 특이점**: 엔드포인트는 `PUT .../drafts`이지만, 실제 동작은 `Response` 엔티티 입장에서는 **부분 수정(Update)**입니다. 하지만 "임시 저장소(drafts)"라는 개념적 리소스 입장에서는 클라이언트가 보낸 상태로 그 질문들의 상태를 **교체(Overwrite)**한다고 볼 수 있습니다.

## 2. `unwrap_or_default()`

`Option<T>` 타입에서 값이 있으면(`Some`) 꺼내고, 없으면(`None`) 해당 타입의 기본값(`Default`)을 반환하는 메서드.

```rust
let content = draft.content.clone().unwrap_or_default();
```

- `String`의 기본값은 빈 문자열 `""`입니다.
- `content`가 `Some("text")`이면 `"text"` 반환
- `content`가 `None`이면 `""` 반환
- Null Safety를 보장하며 코드를 간결하게 만듭니다.

## 3. `Into` trait (타입 변환)

`Model`을 `ActiveModel`로 변환할 때 `into()`를 사용합니다.

```rust
let mut active: response::ActiveModel = response_model.clone().into();
```

- `From` trait을 구현하면 `Into`는 자동으로 구현됩니다.
- SeaORM은 `From<Model> for ActiveModel`을 자동 생성해주므로, `into()` 호출로 쉽게 변환 가능합니다.
- `into()`는 타입을 추론할 수 있어야 하므로, 변수 선언 시 타입을 명시하거나(`: response::ActiveModel`) `response::ActiveModel::from(...)`을 써야 합니다.

## 4. `lock_exclusive()` (행 잠금) - *API-017 참조*

이 API(임시 저장)에서는 사용하지 않지만, 유사한 `API-017`(최종 제출)에서는 동시성 제어를 위해 **행 잠금(Row Lock)**을 사용합니다.

```rust
member_retro::Entity::find()
    .lock_exclusive() // SELECT ... FOR UPDATE
    .one(&txn)
```

- 임시 저장은 단순히 덮어쓰기이므로 락이 필수는 아니지만, 최종 제출은 상태 변경(`DRAFT` -> `SUBMITTED`)이 수반되므로 중복 제출을 막기 위해 락이 필요합니다.

## 5. `usize` vs `i32` (인덱싱)

Rust에서 배열/벡터의 인덱스는 반드시 `usize` 타입이어야 합니다.

```rust
let idx = (draft.question_number - 1) as usize;
let response_model = &responses[idx];
```

- DTO의 `question_number`는 `i32`입니다.
- 이를 인덱스로 쓰기 위해 `as usize`로 캐스팅합니다.
- Rust는 타입 안정성을 위해 숫자 타입 간 암시적 변환을 허용하지 않습니다.

## 6. `chars().count()` vs `len()` (문자열 길이 검증)

```rust
if content.chars().count() > 1000 {
    return Err(AppError::RetroAnswerTooLong(...));
}
```

- `len()`은 바이트 수를 반환합니다 (UTF-8에서 한글은 3바이트).
- `chars().count()`는 유니코드 문자 수를 반환합니다.
- 한국어 서비스이므로 사용자 입장의 "글자 수"를 정확히 세기 위해 `chars().count()`를 사용합니다.

## 7. `NaiveDateTime` (타임존 없는 날짜/시간)

```rust
active.updated_at = Set(now);  // NaiveDateTime 타입
```

- `Utc::now().naive_utc()`는 `NaiveDateTime`을 반환합니다 (타임존 정보 없음).
- DB에는 타임존 없이 UTC 시간을 저장하고, 응답에서는 KST(+9시간)로 변환합니다.
- SeaORM의 `DateTime` 컬럼은 `NaiveDateTime`에 매핑됩니다.

## 8. `AuthUser.user_id()` 헬퍼 메서드

```rust
let user_id = user.user_id()?;
```

- JWT의 `sub` 클레임(문자열)을 `i64`로 파싱하는 헬퍼 메서드입니다.
- 파싱 실패 시 `AppError::Unauthorized`를 반환합니다.
- `src/utils/auth.rs`에 정의되어 있으며, `user.0.sub.parse()`의 간결한 래퍼입니다.