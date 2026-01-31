# Spring 개발자를 위한 Rust API 구현 가이드 (API-022)

이 문서는 **API-022 (회고 종합 분석)**의 Rust 구현을 Spring(Java/Kotlin) 개발자의 관점에서 이해하기 쉽게 설명합니다.
API-022는 **AI 분석**과 **트랜잭션 처리**, **복잡한 데이터 집계**가 포함된 비즈니스 로직을 담고 있어, Rust의 서비스 계층 패턴을 심도 있게 이해할 수 있습니다.

## 1. 아키텍처 매핑

Spring의 계층형 아키텍처와 유사하며, 외부 AI 서비스 호출이 포함된 구조입니다.

| 역할 | Rust (현재 프로젝트) | Spring (JVM) | 비고 |
|------|-------------------|--------------|------|
| **Web Layer** | `handler.rs` | `Controller` | HTTP 요청 처리 및 검증 |
| **Service Layer** | `service.rs` | `Service` | 비즈니스 로직 (데이터 집계, 트랜잭션 관리) |
| **AI Client** | `AiService` (Trait/Impl) | `FeignClient` / `RestTemplate` | 외부 AI API 호출 추상화 |
| **Persistence** | `SeaORM` | `JPA / QueryDSL` | DB CRUD |
| **Transaction** | `txn.commit()` | `@Transactional` | 트랜잭션 제어 |

---

## 2. 코드 상세 분석

### 2.1 Handler Layer (`handler.rs`)

**Rust 코드 (실제 구현: `handler.rs:410-436`):**
```rust
// POST /api/v1/retrospects/{retrospectId}/analysis
pub async fn analyze_retrospective_handler(
    user: AuthUser,                    // 1. 인증 정보 (JWT 미들웨어)
    State(state): State<AppState>,     // 2. DI (Service, AI Client 포함)
    Path(retrospect_id): Path<i64>,    // 3. Path Variable
) -> Result<Json<BaseResponse<AnalysisResponse>>, AppError> {

    // retrospectId 검증 (1 이상의 양수)
    if retrospect_id < 1 {
        return Err(AppError::BadRequest("retrospectId는 1 이상의 양수여야 합니다.".to_string()));
    }

    // 사용자 ID 추출 (주의: 다른 핸들러와 달리 user.user_id()가 아닌 직접 파싱)
    let user_id: i64 = user.0.sub.parse()
        .map_err(|_| AppError::Unauthorized("유효하지 않은 사용자 ID입니다.".to_string()))?;

    // 서비스 호출
    let result = RetrospectService::analyze_retrospective(state, user_id, retrospect_id).await?;

    Ok(Json(BaseResponse::success_with_message(result, "회고 분석이 성공적으로 완료되었습니다.")))
}
```

**Spring 관점 해석:**

*   **`State(state)`**: Spring의 `ApplicationContext`와 유사합니다. `state.ai_service`를 통해 AI 클라이언트에 접근합니다.
*   **`user.0.sub.parse()`**: Spring에서 `@AuthenticationPrincipal`로 받은 사용자 정보의 ID를 추출하는 것과 유사합니다. 이 핸들러는 `user.user_id()` 헬퍼 메서드 대신 직접 파싱하는데, 같은 파일의 다른 핸들러들과 일관성이 다른 부분입니다.
*   **비동기 처리**: `async/await` 패턴을 사용하여 I/O (DB, 외부 API) 작업 중 스레드를 차단하지 않습니다. Spring WebFlux와 유사하지만 코드는 동기식처럼 작성됩니다.

---

### 2.2 Service Layer (`service.rs`) - 핵심 로직

이 API의 핵심 비즈니스 로직은 다음과 같은 순서로 진행됩니다.

#### 1. 사전 검증 (Validations)
```rust
// 1. 회고 존재 여부 & 이미 분석됐는지 확인
let retrospect = find_retrospect(...)?;
if retrospect.team_insight.is_some() {
    return Err(AppError::RetroAlreadyAnalyzed(...));
}

// 2. 권한 확인 (팀 멤버인지)
if !is_team_member { return Err(AppError::TeamAccessDenied(...)); }

// 3. 월간 한도 체크 (이번 달 분석 횟수 < 10)
let count = count_monthly_analysis(...).await?;
if count >= 10 { return Err(AppError::AiMonthlyLimitExceeded(...)); }

// 4. 데이터 충분성 체크 (답변 수, 참여자 수)
if submitted_members.is_empty() || answer_count < 3 {
    return Err(AppError::RetroInsufficientData(...));
}
```
*   **Spring 비교**: `Service` 메소드 초입의 `Guard Clauses`와 동일합니다. 예외 발생 시 즉시 리턴하여 불필요한 연산을 방지합니다.

#### 2. 데이터 집계 (Data Aggregation)
AI에게 보낼 데이터를 만들기 위해 여러 테이블을 조회하고 조합합니다.

```rust
// 참여자 목록 조회
let members = member::Entity::find()...all().await?;

// 답변 데이터 수집 (Member -> Answers 매핑)
let mut members_data = Vec::new();
for member in members {
    // ... 답변 필터링 및 매핑 로직 ...
    members_data.push(MemberAnswerData { ... });
}
```
*   **Spring 비교**: JPA로 `Member`와 `Response`를 `Fetch Join`하여 가져오거나, 각각 조회 후 Java Stream API로 `Map<Member, List<Response>>`를 만드는 과정과 같습니다. Rust의 `Iterator` 체이닝(`map`, `filter`, `collect`)이 Java Stream과 매우 유사하게 사용됩니다.

#### 3. AI 서비스 호출 (External API Call)
```rust
// 8. AI 서비스 호출 (service.rs:1784-1787)
let mut analysis = state.ai_service
    .analyze_retrospective(&members_data) // 데이터 전달
    .await?; // 응답 대기

// personalMissions의 userId 오름차순 정렬 (service.rs:1790)
analysis.personal_missions.sort_by_key(|pm| pm.user_id);
```
*   **Spring 비교**: `OpenAiService.analyze(data)`와 같이 외부 컴포넌트를 호출하는 부분입니다. `State`에 주입된 구현체를 사용하므로 테스트 시 Mocking이 가능합니다.
*   **정렬**: AI가 반환한 `personalMissions` 배열은 `userId` 오름차순으로 재정렬됩니다. AI 출력 순서를 신뢰하지 않고 코드에서 보장하는 방어적 설계입니다.

#### 4. 트랜잭션 처리 (Transaction Management)
분석 결과를 DB에 저장합니다. 여러 테이블(회고, 멤버별 결과)을 동시에 업데이트해야 하므로 트랜잭션이 필수입니다.

```rust
// 트랜잭션 시작
let txn = state.db.begin().await?;

// 1. 회고(Retrospect) 테이블 업데이트 (팀 인사이트)
let mut retro_active: ActiveModel = retrospect_model.into();
retro_active.team_insight = Set(Some(analysis.team_insight));
retro_active.update(&txn).await?;

// 2. 멤버_회고(MemberRetro) 테이블 업데이트 (개인별 인사이트, 상태 변경)
for member in submitted_members {
    let mut mr_active: ActiveModel = member.into();
    mr_active.personal_insight = Set(Some(...));
    mr_active.status = Set(RetrospectStatus::Analyzed);
    mr_active.update(&txn).await?;
}

// 커밋
txn.commit().await?;
```

**Spring과 가장 다른 점 (SeaORM vs JPA):**
*   **Spring (JPA)**: `@Transactional` 어노테이션을 붙이면 메소드 종료 시 변경 감지(Dirty Checking)가 동작하여 자동으로 `UPDATE` 쿼리가 나갑니다.
*   **Rust (SeaORM)**:
    1.  `db.begin()`으로 트랜잭션을 명시적으로 시작합니다.
    2.  `ActiveModel` 객체의 필드를 `Set(...)`으로 수정합니다.
    3.  `.update(&txn)` 메소드를 호출하여 **직접 쿼리를 실행**해야 합니다.
    4.  `txn.commit()`으로 확정합니다.

이 방식은 코드가 다소 길어지지만, **어떤 시점에 어떤 쿼리가 나가는지 명확하게 제어**할 수 있다는 장점이 있습니다.

---

## 3. 핵심 요약

API-022는 **복합적인 비즈니스 로직**을 다루는 전형적인 서비스 메소드입니다.

1.  **유효성 검증**: 정책(월간 한도, 최소 데이터)에 따른 방어 로직
2.  **데이터 가공**: DB 조회 결과를 AI 모델 입력 포맷으로 변환
3.  **외부 연동**: AI API 호출 (비동기 I/O)
4.  **원자성 보장**: 트랜잭션을 통한 일관된 데이터 저장

Spring 개발자라면 **"JPA의 영속성 컨텍스트가 없어서 `update()`를 직접 호출해야 한다"**는 점과 **"예외 처리 대신 `Result` 타입을 리턴한다"**는 점만 유의하면, 전체적인 흐름은 매우 익숙하게 느끼실 것입니다.

---

## 4. 스펙 vs 구현 차이 (참고)

API 스펙 문서와 실제 구현 사이에 에러 코드 차이가 있습니다.

*   **데이터 부족 에러**: 스펙은 `RETRO4042` (404)이지만, 구현은 `RETRO4221` (422 Unprocessable Entity)입니다. Spring에서는 `@ResponseStatus(HttpStatus.UNPROCESSABLE_ENTITY)`에 해당합니다.
*   **AI 에러 세분화**: 스펙은 `AI5001` 하나이지만, 구현은 `AI5001`/`AI5002`/`AI5031`/`AI5003`으로 세분화됩니다. Spring의 `@ExceptionHandler`에서 예외 타입별로 다른 HTTP 상태를 반환하는 것과 같습니다.

자세한 내용은 [README.md](./README.md)의 "스펙 문서 vs 실제 구현 차이점" 섹션을 참고하세요.
