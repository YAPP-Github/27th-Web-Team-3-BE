# [API-019] 보관함 조회 - 동작 흐름

## 전체 흐름 요약

```
HTTP Request
  GET /api/v1/retrospects/storage?range=3_MONTHS
  Authorization: Bearer {token}
        │
        ▼
  ┌─────────────────────────────────────────────────┐
  │  1. Query 파라미터 파싱                           │
  │     Query(params): Query<StorageQueryParams>     │
  │     → ?range=3_MONTHS → StorageRangeFilter::ThreeMonths │
  │     → ?range 생략     → StorageQueryParams { range: None } │
  │     → ?range=INVALID  → 400 Bad Request (serde 역직렬화 실패) │
  │                                                  │
  │     소스: handler.rs:371                          │
  └───────────────────┬─────────────────────────────┘
                      │
                      ▼
  ┌─────────────────────────────────────────────────┐
  │  2. 기간 필터 기본값 처리                         │
  │     let range_filter = params.range              │
  │         .unwrap_or_default();                    │
  │                                                  │
  │     None → StorageRangeFilter::All (#[default])  │
  │     Some(filter) → 그대로 사용                    │
  │                                                  │
  │     소스: service.rs:693                          │
  └───────────────────┬─────────────────────────────┘
                      │
                      ▼
  ┌─────────────────────────────────────────────────┐
  │  3. 제출 완료된 회고만 조회                       │
  │                                                  │
  │     member_retro::Entity::find()                 │
  │       .filter(MemberId.eq(user_id))              │
  │       .filter(Status.is_in([                     │
  │           Submitted, Analyzed                    │
  │       ]))                                        │
  │                                                  │
  │     소스: service.rs:702-707                      │
  └───────────────────┬─────────────────────────────┘
                      │
                      ▼
  ┌─────────────────────────────────────────────────┐
  │  4. 기간 필터 적용                                │
  │                                                  │
  │     if let Some(days) = range_filter.days() {    │
  │         cutoff = Utc::now().naive_utc()           │
  │                  - Duration::days(days);          │
  │         query.filter(SubmittedAt.gte(cutoff));    │
  │     }                                            │
  │                                                  │
  │     ALL        → days() = None → 필터 안 함       │
  │     3_MONTHS   → days() = Some(90)  → 90일 전부터 │
  │     6_MONTHS   → days() = Some(180) → 180일 전부터│
  │     1_YEAR     → days() = Some(365) → 365일 전부터│
  │                                                  │
  │     소스: service.rs:710-714                      │
  └───────────────────┬─────────────────────────────┘
                      │
                      ▼
  ┌─────────────────────────────────────────────────┐
  │  5. 빈 결과 조기 반환                             │
  │                                                  │
  │     if member_retros.is_empty() {                │
  │         return Ok(StorageResponse {              │
  │             years: vec![]                        │
  │         });                                      │
  │     }                                            │
  │                                                  │
  │     소스: service.rs:721-723                      │
  └───────────────────┬─────────────────────────────┘
                      │ (결과가 있는 경우)
                      ▼
  ┌─────────────────────────────────────────────────┐
  │  6. 관련 데이터 추가 조회                         │
  │                                                  │
  │     a) 회고 정보 조회                             │
  │        retrospect::Entity::find()                │
  │          .filter(RetrospectId.is_in(ids))        │
  │        소스: service.rs:729-733                   │
  │                                                  │
  │     b) 참여자 수 배치 조회                        │
  │        member_retro::Entity::find()              │
  │          .filter(RetrospectId.is_in(ids))        │
  │        → HashMap<retrospect_id, count> 집계      │
  │        소스: service.rs:736-745                   │
  └───────────────────┬─────────────────────────────┘
                      │
                      ▼
  ┌─────────────────────────────────────────────────┐
  │  7. 연도별 그룹화 (BTreeMap)                     │
  │                                                  │
  │     BTreeMap<i32, Vec<StorageRetrospectItem>>    │
  │                                                  │
  │     for retro in &retrospects {                  │
  │         // UTC → KST 변환 (+9시간)               │
  │         // submitted_at 기준 연도 추출            │
  │         // year_groups.entry(year)               │
  │         //     .or_default().push(item);         │
  │     }                                            │
  │                                                  │
  │     소스: service.rs:748-784                      │
  └───────────────────┬─────────────────────────────┘
                      │
                      ▼
  ┌─────────────────────────────────────────────────┐
  │  8. 정렬                                         │
  │                                                  │
  │     a) BTreeMap.into_iter().rev()                │
  │        → 연도 내림차순 (2026 → 2025 → 2024)      │
  │                                                  │
  │     b) items.sort_by(|a, b|                      │
  │            b.display_date.cmp(&a.display_date))  │
  │        → 각 연도 그룹 내 최신순 정렬              │
  │                                                  │
  │     c) years.sort_by() 보조 정렬                  │
  │        → BTreeMap rev() 결과 안전성 보장          │
  │                                                  │
  │     소스: service.rs:788-802                      │
  └───────────────────┬─────────────────────────────┘
                      │
                      ▼
  ┌─────────────────────────────────────────────────┐
  │  9. 응답 반환                                    │
  │                                                  │
  │     Ok(StorageResponse { years })                │
  │     → BaseResponse::success_with_message(...)    │
  │                                                  │
  │     소스: service.rs:804, handler.rs:379-382      │
  └─────────────────────────────────────────────────┘
```

## DB 쿼리 흐름

총 3회의 DB 쿼리가 발생한다.

```
┌──────────────────────────────────────────────┐
│  Query 1: 사용자의 제출 완료 회고 목록          │
│                                              │
│  SELECT * FROM member_retro                  │
│  WHERE member_id = ?                         │
│    AND status IN ('SUBMITTED', 'ANALYZED')   │
│    AND submitted_at >= ?   -- 기간 필터 시     │
│                                              │
│  소스: service.rs:702-719                     │
└──────────────────┬───────────────────────────┘
                   │ retrospect_ids 추출
                   ▼
┌──────────────────────────────────────────────┐
│  Query 2: 회고 상세 정보 조회                   │
│                                              │
│  SELECT * FROM retrospect                    │
│  WHERE retrospect_id IN (?, ?, ...)          │
│                                              │
│  소스: service.rs:729-733                     │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│  Query 3: 참여자 수 배치 조회                   │
│                                              │
│  SELECT * FROM member_retro                  │
│  WHERE retrospect_id IN (?, ?, ...)          │
│                                              │
│  → 어플리케이션 레벨에서 HashMap 집계           │
│                                              │
│  소스: service.rs:736-745                     │
└──────────────────────────────────────────────┘
```

## 날짜 처리 흐름

```
submitted_at (DB: NaiveDateTime, UTC 기준)
        │
        ▼
  + Duration::hours(9)  ← UTC → KST 변환
        │
        ├──► display_date: "2026-01-24"  (format "%Y-%m-%d")
        │    소스: service.rs:760-767
        │
        └──► year: 2026  (format "%Y" → parse::<i32>())
             소스: service.rs:769-774
             → BTreeMap의 key로 사용

  [fallback] submitted_at이 None인 경우:
             retro.created_at + KST offset 사용
             소스: service.rs:763-767, 772
```

## 에러 흐름

```
1. 쿼리 파라미터 파싱 실패
   ?range=INVALID
   → serde 역직렬화 실패
   → Axum이 400 Bad Request 자동 반환

2. JWT 인증 실패
   → AuthUser extractor 실패
   → 401 Unauthorized

3. DB 조회 오류
   → SeaORM DbErr
   → .map_err(|e| AppError::InternalError(e.to_string()))
   → 500 Internal Server Error
   소스: service.rs:719, 733, 740
```
