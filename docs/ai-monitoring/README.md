# AI 자율 모니터링 시스템

## 개요

로그 기반 AI 자율 모니터링 시스템으로, 이상 징후 감지부터 자동 이슈 생성 및 리팩토링 PR 제출까지 자동화합니다.

```
[로그 수집] → [AI 분석] → [Discord 알림] → [GitHub Issue] → [자동 수정 PR]
```

## 핵심 가치

| 지표 | 현재 | 목표 |
|------|------|------|
| 장애 감지 시간 | 수동 확인 (수 분~수 시간) | 실시간 (< 1분) |
| 초기 진단 시간 | 개발자 직접 분석 (30분+) | AI 자동 진단 (< 5분) |
| 반복 이슈 해결 | 매번 수동 | 패턴 학습 후 자동 제안 |

## 폴더 구조

```
docs/ai-monitoring/
├── README.md              # 프로젝트 개요 (현재 문서)
├── pipeline-guide.md      # 🆕 운영 가이드 (파이프라인 상세)
├── MOC.md                 # 문서 지도
├── design/                # 설계 문서
│   ├── 00-tool-comparison.md
│   ├── 01-architecture.md
│   ├── 02-log-specification.md
│   ├── 03-agents.md
│   └── 04-alerting.md
└── phases/                # 구축 가이드
    ├── 05-implementation-plan.md  # 로드맵 개요
    ├── phase-1-log-foundation.md
    ├── phase-2-monitoring-mvp.md
    ├── phase-3-ai-diagnostic.md
    └── phase-4-automation.md
```

## 문서 구조

### 설계 문서 (`design/`)
| 문서 | 설명 |
|------|------|
| [00-tool-comparison.md](./design/00-tool-comparison.md) | 모니터링 도구 비교 및 선택 이유 |
| [01-architecture.md](./design/01-architecture.md) | 시스템 아키텍처 및 데이터 흐름 |
| [02-log-specification.md](./design/02-log-specification.md) | 로그 포맷 및 수집 스펙 |
| [03-agents.md](./design/03-agents.md) | AI Agent 설계 및 역할 |
| [04-alerting.md](./design/04-alerting.md) | Discord 알림 및 GitHub 연동 |

### 구축 가이드 (`phases/`)
| Phase | 이름 | 문서 | 내용 | 기간 |
|-------|------|------|------|------|
| 개요 | - | [05-implementation-plan.md](./phases/05-implementation-plan.md) | 전체 로드맵 및 태스크 | - |
| 1 | Foundation | [phase-1-log-foundation.md](./phases/phase-1-log-foundation.md) | 로그 기반 구축 (JSON, 에러 코드, Request ID) | Week 1-2 |
| 2 | MVP | [phase-2-monitoring-mvp.md](./phases/phase-2-monitoring-mvp.md) | 모니터링 MVP (Discord, Log Watcher, Cron) | Week 3-4 |
| 3 | AI | [phase-3-ai-diagnostic.md](./phases/phase-3-ai-diagnostic.md) | AI 진단 연동 (Claude API, 컨텍스트 수집) | Week 5-6 |
| 4 | Production | [phase-4-automation.md](./phases/phase-4-automation.md) | 자동화 확장 (GitHub Issue, Auto-Fix PR) | Week 7-8 |

## 시스템 구성도 (간략)

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Rust Server │────▶│  Log Store   │────▶│  AI Monitor  │
│  (tracing)   │     │  (Loki/File) │     │  (Claude)    │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
                     ┌────────────────────────────┼────────────────────────────┐
                     ▼                            ▼                            ▼
              ┌──────────────┐            ┌──────────────┐            ┌──────────────┐
              │   Discord    │            │    GitHub    │            │   Auto-Fix   │
              │   Webhook    │            │    Issue     │            │   PR Agent   │
              └──────────────┘            └──────────────┘            └──────────────┘
```

## 운영 가이드

> **[pipeline-guide.md](./pipeline-guide.md)** - 현재 운영 중인 파이프라인 상세 가이드

| 항목 | 상태 |
|------|------|
| 로그 감시 (Cron) | ✅ 운영 중 |
| AI 진단 (OpenAI) | ✅ 운영 중 |
| Discord 알림 | ✅ 운영 중 |
| GitHub Issue 자동 생성 | ✅ 운영 중 |
| Auto-Fix PR | 🔄 대기 (auto_fixable 에러 시) |

---

## 빠른 시작

### 1. 로그 확인
```bash
# 현재 로그 포맷 확인
cd codes/server && cargo run 2>&1 | head -20
```

### 2. Discord Webhook 설정
```bash
# .env에 추가
DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/...
```

### 3. 모니터링 Agent 실행
```bash
# (구현 후) 모니터링 시작
./scripts/log-watcher.sh
```

## 기술 스택

| 구성요소 | 기술 | 상태 |
|----------|------|------|
| 로깅 | `tracing` + JSON formatter | ✅ 운영 중 |
| 로그 저장 | File (MVP) → Loki (추후) | ✅ 운영 중 |
| AI Agent | OpenAI GPT-4o-mini | ✅ 운영 중 |
| 알림 | Discord Webhook | ✅ 운영 중 |
| Issue 생성 | GitHub CLI | ✅ 운영 중 |
| 자동화 | Shell Scripts + Cron | ✅ 운영 중 |

## 비용 구조

| 영역 | MVP | Production | 비용 | 선택 근거 |
|------|-----|------------|------|-----------|
| 로그 | 파일 | Loki | $0 | ELK 과도함, CloudWatch 비용 누적 |
| 메트릭 | - | Prometheus | $0 | CNCF 표준, Grafana 네이티브 통합 |
| 알림 | Discord | Discord | $0 | 팀 사용 중, Webhook 설정 5분 |
| AI | Shell Script | Claude Agent | ~$30/월 | 컨텍스트 기반 진단, 자동 수정 가능 |
| 시각화 | - | Grafana | $0 | Loki/Prometheus 동일 생태계 |

**MVP: $0** | **Production: ~$30/월** (Claude API 호출 비용만)

> 상세 선택 근거는 [00-tool-comparison.md](./design/00-tool-comparison.md#선택-근거) 참조

## 관련 링크

- [Tracing 문서](https://docs.rs/tracing)
- [Discord Webhook 가이드](https://discord.com/developers/docs/resources/webhook)
- [Claude API 문서](https://docs.anthropic.com/claude/reference)
