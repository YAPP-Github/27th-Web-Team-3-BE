# Payment Engine 1-Pager

## 프로젝트 정보

| 항목 | 값 |
|------|-----|
| **프로젝트명** | FluxPay |
| **GitHub Repository** | `fluxpay-engine` |
| **현재 적용 서비스** | 모아 (피벗 예정) |
| **목적** | 도메인 독립적인 결제/과금 엔진 |
| **포지셔닝** | 분산 환경에서 데이터 무결성을 보장하는 플러그인형 결제 플랫폼 |

---

## 프로젝트 개요

어떤 서비스 앞에도 붙여서 **결제, 과금, 트래픽 제어**를 처리할 수 있는 재사용 가능한 엔진.
특정 도메인에 종속되지 않고, 설정 기반으로 다양한 비즈니스 모델(크레딧, 구독, 종량제)을 지원한다.

---

## 비즈니스 컨텍스트

### 해결하려는 문제
- 유료 기능 도입 시마다 결제 시스템을 새로 구축하는 비효율
- 분산 환경에서 결제-서비스 간 데이터 정합성 보장 어려움
- 트래픽 폭주(선착순 이벤트) 시 시스템 보호 메커니즘 부재

### 목표
- 플러그인처럼 기존 서비스에 연결 가능한 결제 엔진
- 장애 상황에서도 데이터 일관성 100% 보장
- 트래픽 제어를 통한 하위 시스템 보호

---

## 기술 스택

| 레이어 | 기술 | 선택 이유 |
|--------|------|----------|
| Framework | Spring Boot 3 + WebFlux | 비동기/논블로킹, 생태계 |
| Language | Kotlin + Coroutines | 간결한 비동기 코드 |
| Database | PostgreSQL | 트랜잭션 안정성 |
| Message Queue | Kafka | 이벤트 소싱, 내구성 |
| Cache | Redis | Lua Script 기반 원자적 연산 |
| PG 연동 | 토스페이먼츠 | 국내 표준, 문서화 우수 |

---

## 시스템 아키텍처

```
[Client App]
    │
    ▼
┌─────────────────────────────────────────────────┐
│              FluxPay Engine                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │
│  │   Gateway   │  │   Payment   │  │  Queue  │ │
│  │ (대기열/제어)│  │  (결제/과금) │  │ (Kafka) │ │
│  └─────────────┘  └─────────────┘  └─────────┘ │
└────────────────────────┬────────────────────────┘
                         │ Event-Driven
         ┌───────────────┼───────────────┐
         ▼               ▼               ▼
   [Service A]     [Service B]     [Service N]
   (any backend)   (any backend)   (any backend)
```

### 연동 방식
- **Inbound:** REST API (결제 요청, 잔액 조회)
- **Outbound:** Kafka 이벤트 (결제 완료, 크레딧 차감 알림)
- **설정:** YAML 기반 서비스별 과금 정책 정의

---

## 핵심 도메인

### 1. 주문 (Order)
- 유료 기능 사용 요청 → 주문 생성
- 상태: `PENDING` → `PAID` → `COMPLETED` / `CANCELLED`
- 서비스 독립적: 주문 메타데이터로 도메인 정보 전달

### 2. 결제 (Payment)
- PG사 연동 (토스페이먼츠, 확장 가능)
- 결제 수단: 카드, 간편결제
- 상태: `READY` → `APPROVED` → `CONFIRMED` / `FAILED`

### 3. 크레딧 (Credit)
- 선불 충전 방식 (포인트, 토큰)
- 사용량 차감 및 환불 처리
- 서비스별 차등 과금 정책 지원

### 4. 구독 (Subscription)
- 월정액/연정액 구독 모델
- 자동 갱신, 해지, 다운그레이드
- 플랜별 기능 제한 설정

---

## 기술적 도전 과제

### 1. Saga 패턴 (Orchestration)
**문제:** 서비스 호출 성공 → 결제 실패 시 정합성 불일치
**해결:** 보상 트랜잭션으로 자동 롤백

```
[정상 플로우]
주문생성 → 결제승인 → 서비스실행 → 크레딧차감 → 완료

[실패 시 보상]
서비스실패 → 결제취소 → 주문취소 → 크레딧복구
```

### 2. Transactional Outbox
**문제:** DB 커밋 성공 + Kafka 발행 실패 → 이벤트 유실
**해결:** Outbox 테이블에 이벤트 저장 → Polling Publisher가 Kafka로 발행

### 3. 멱등성 보장
**문제:** 네트워크 재시도로 중복 결제 발생
**해결:** Redis Lua Script로 요청 ID 기반 원자적 중복 체크

### 4. 트래픽 제어 (Virtual Waiting Room)
**문제:** 선착순 이벤트, 대규모 프로모션 시 서버 과부하
**해결:** Redis Sorted Set 기반 대기열 + 동적 입장 속도 조절

---

## API 설계 원칙

### 요청/응답 형식
- Request: `camelCase` JSON
- Response: 표준 응답 래퍼 (`isSuccess`, `code`, `message`, `result`)

### 에러 코드 체계
| 코드 | 설명 |
|------|------|
| PAY_001 | 잔액 부족 |
| PAY_002 | 결제 승인 실패 |
| PAY_003 | 이미 처리된 요청 |
| PAY_004 | 주문 상태 불일치 |

### 주요 엔드포인트
- `POST /api/v1/orders` - 주문 생성
- `POST /api/v1/payments/confirm` - 결제 승인
- `GET /api/v1/credits/balance` - 잔액 조회
- `POST /api/v1/credits/use` - 크레딧 사용

---

## 클라이언트 서비스 연동

### 이벤트 발행 (Client → FluxPay)
- `service.usage.requested` - 유료 기능 사용 요청
- `service.usage.completed` - 기능 실행 완료
- `service.usage.failed` - 기능 실행 실패

### 이벤트 구독 (FluxPay → Client)
- `payment.approved` - 결제 승인됨
- `payment.failed` - 결제 실패
- `credit.deducted` - 크레딧 차감됨
- `credit.refunded` - 크레딧 환불됨
- `subscription.activated` - 구독 활성화
- `subscription.expired` - 구독 만료

### SDK 제공 (예정)
- Java/Kotlin SDK
- Node.js SDK
- Rust SDK

---

## 구현 우선순위

### Phase 1: Core Payment
- [ ] 프로젝트 셋업 (Spring Boot 3, Kotlin, WebFlux)
- [ ] 주문/결제 도메인 구현
- [ ] 토스페이먼츠 연동
- [ ] 기본 크레딧 시스템
- [ ] REST API 및 Swagger 문서화

### Phase 2: Event-Driven Architecture
- [ ] Kafka 연동 및 이벤트 스키마 정의
- [ ] Transactional Outbox 패턴 구현
- [ ] Saga Orchestrator 구현
- [ ] 클라이언트 서비스 연동 테스트

### Phase 3: Traffic Control & Subscription
- [ ] Redis 대기열 시스템 (Virtual Waiting Room)
- [ ] Rate Limiting 미들웨어
- [ ] 구독 도메인 구현
- [ ] 부하 테스트 (k6/Locust)

---

## 성공 기준

| 지표 | 목표 |
|------|------|
| 결제 성공률 | 99.9% 이상 |
| 중복 결제 | 0건 |
| 데이터 정합성 | 100% (주문-결제-크레딧 일치) |
| 장애 복구 | 자동 보상 트랜잭션 |
| 동시 처리 | 1,000 TPS 이상 |

---

## 제약 조건

- 클라이언트 서비스 코드 수정 최소화 (이벤트 기반 연동)
- Kafka를 통한 느슨한 결합 유지
- 결제 민감 정보 암호화 필수 (AES-256)
- PCI-DSS 가이드라인 준수
- 도메인 로직은 FluxPay 내부에 캡슐화

---

## 기술 키워드

`Spring WebFlux` `Kotlin Coroutines` `Kafka` `Redis Lua Script`
`Saga Pattern` `Transactional Outbox` `Idempotent API`
`Virtual Waiting Room` `Rate Limiting` `Event Sourcing`
