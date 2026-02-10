# Phase 4: Issue Automation

> **버전**: 1.0
> **최종 수정**: 2026-02-06
> **상태**: 구현 대기
> **의존성**: Phase 3 (AI Diagnostic) 완료 필수

---

## 개요

| 항목 | 내용 |
|------|------|
| Phase | 4: Issue Automation |
| 기간 | 3-5일 |
| 목표 | AI 진단 결과 기반 GitHub Issue 자동 생성 및 관리 |
| 선행 조건 | Phase 3 (AI Diagnostic) 완료 |
| 후속 단계 | Phase 5 (Auto-Fix & PR) |

---

## 1. 목표 및 범위

### 1.1 왜 필요한가?

Phase 3까지 구현된 AI 진단 시스템은 에러 분석 결과를 Discord로 알림하지만,
체계적인 이슈 추적이 불가능합니다.

**현재 문제:**
- Discord 알림만으로는 이슈 추적 어려움
- 동일 에러 반복 발생 시 히스토리 파악 불가
- 담당자 할당 및 진행 상태 관리 불가

**Phase 4 해결:**
- GitHub Issue로 체계적 추적
- 중복 에러 자동 감지 및 코멘트 추가
- 우선순위 라벨로 긴급도 구분

### 1.2 목표

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Phase 4 목표                                      │
├─────────────────────────────────────────────────────────────────────────┤
│  1. Critical/High 에러 발생 시 GitHub Issue 자동 생성                    │
│  2. 중복 에러 감지 및 기존 이슈에 코멘트 추가                            │
│  3. 심각도 기반 우선순위 라벨 자동 부여                                  │
│  4. Discord 알림과 Issue 연동                                            │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.3 범위

**포함 (In Scope):**
- GitHub Issue 자동 생성 스크립트
- 중복 이슈 감지 로직
- 우선순위 라벨 시스템
- 기존 이슈 코멘트 추가
- Discord 알림 연동

**제외 (Out of Scope):**
- 자동 코드 수정 (Phase 5)
- Draft PR 생성 (Phase 5)
- 리뷰어 자동 할당 (Phase 5+)

---

## 2. 아키텍처

### 2.1 전체 흐름

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Phase 4 Issue Automation 파이프라인                  │
└─────────────────────────────────────────────────────────────────────────┘

  Phase 3 완료 (Diagnostic 결과)
       │
       ▼
┌──────────────────┐
│ severity 확인    │
└────────┬─────────┘
         │
    ┌────┴────────────┐
    │                 │
    ▼                 ▼
 critical/high      medium/low
    │                 │
    ▼                 ▼
┌───────────┐     ┌───────────┐
│ Issue 생성│     │ 로그만    │
│ 프로세스  │     │ 기록      │
└─────┬─────┘     └───────────┘
      │
      ▼
┌──────────────────┐
│ 중복 체크        │
│ (error_code 검색)│
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
  중복       신규
    │         │
    ▼         ▼
┌───────┐   ┌───────────┐
│코멘트 │   │ 새 Issue  │
│추가   │   │ 생성      │
└───────┘   └─────┬─────┘
                  │
                  ▼
            ┌───────────┐
            │ 라벨 부여 │
            │ Discord   │
            │ 알림      │
            └───────────┘
```

### 2.2 컴포넌트 구조

```
scripts/
├── log-watcher.sh          # Phase 2 (기존)
├── discord-alert.sh        # Phase 2 (기존)
├── diagnostic-agent.py     # Phase 3 (기존)
├── create-issue.sh         # Phase 4 (신규) ⬅️
└── setup-labels.sh         # Phase 4 (신규) ⬅️
```

---

## 3. 구현 상세

### 3.1 라벨 초기 설정

**파일**: `scripts/setup-labels.sh`

```bash
#!/bin/bash
set -euo pipefail

echo "GitHub 라벨 생성 중..."

# AI 자동화 라벨
gh label create "ai-generated" \
    --color "7057ff" \
    --description "AI 모니터링 시스템이 자동 생성" \
    --force

# 우선순위 라벨
gh label create "priority:critical" \
    --color "b60205" \
    --description "즉시 대응 필요 (P0)" \
    --force

gh label create "priority:high" \
    --color "d93f0b" \
    --description "우선 대응 필요 (P1)" \
    --force

gh label create "priority:medium" \
    --color "fbca04" \
    --description "일반 우선순위 (P2)" \
    --force

gh label create "priority:low" \
    --color "0e8a16" \
    --description "낮은 우선순위 (P3)" \
    --force

# 도메인 라벨
gh label create "domain:ai" \
    --color "1d76db" \
    --description "AI/LLM 관련 에러" \
    --force

gh label create "domain:auth" \
    --color "5319e7" \
    --description "인증/인가 관련 에러" \
    --force

gh label create "domain:db" \
    --color "0052cc" \
    --description "데이터베이스 관련 에러" \
    --force

echo "라벨 생성 완료!"
```

### 3.2 Issue 자동 생성

**파일**: `scripts/create-issue.sh`

```bash
#!/bin/bash
set -euo pipefail

#######################################
# GitHub Issue 자동 생성 스크립트
# Phase 4: Issue Automation
#######################################

# 입력 검증
if [ -z "${1:-}" ]; then
    echo "Usage: $0 <diagnostic_json>"
    exit 1
fi

DIAGNOSTIC_JSON="$1"

# JSON 파싱
ERROR_CODE=$(echo "$DIAGNOSTIC_JSON" | jq -r '.error_code // "UNKNOWN"')
SEVERITY=$(echo "$DIAGNOSTIC_JSON" | jq -r '.severity // "medium"')
ROOT_CAUSE=$(echo "$DIAGNOSTIC_JSON" | jq -r '.root_cause // "원인 분석 중"')
IMPACT=$(echo "$DIAGNOSTIC_JSON" | jq -r '.impact // "영향 분석 중"')
AUTO_FIXABLE=$(echo "$DIAGNOSTIC_JSON" | jq -r '.auto_fixable // false')
RECOMMENDATIONS=$(echo "$DIAGNOSTIC_JSON" | jq -r '
    if .recommendations then
        .recommendations | map("- " + .action) | join("\n")
    else
        "- 추가 분석 필요"
    end
')

# 심각도 체크: medium/low는 Issue 생성 안 함
if [[ "$SEVERITY" != "critical" && "$SEVERITY" != "high" ]]; then
    echo "심각도 $SEVERITY: Issue 생성 생략 (critical/high만 대상)"
    exit 0
fi

echo "=== Issue 생성 프로세스 시작 ==="
echo "에러 코드: $ERROR_CODE"
echo "심각도: $SEVERITY"

#######################################
# 중복 체크
#######################################
echo "중복 이슈 확인 중..."

EXISTING_ISSUE=$(gh issue list \
    --label "ai-generated" \
    --state open \
    --search "in:title $ERROR_CODE" \
    --json number,title \
    --jq '.[0].number // empty' 2>/dev/null || echo "")

if [ -n "$EXISTING_ISSUE" ]; then
    echo "기존 이슈 #$EXISTING_ISSUE 발견. 코멘트 추가..."

    gh issue comment "$EXISTING_ISSUE" --body "$(cat <<EOF
## 에러 재발생 감지

| 항목 | 내용 |
|------|------|
| 발생 시간 | $(date -u +"%Y-%m-%dT%H:%M:%SZ") |
| 에러 코드 | \`$ERROR_CODE\` |
| 심각도 | $SEVERITY |

### 근본 원인 (재분석)
$ROOT_CAUSE

### 영향 범위
$IMPACT

---
*AI Monitor 자동 코멘트*
EOF
)"

    echo "코멘트 추가 완료: Issue #$EXISTING_ISSUE"

    # Discord 알림 (재발생)
    if [ -f "./scripts/discord-alert.sh" ]; then
        ./scripts/discord-alert.sh "warning" "에러 재발생" \
            "에러 코드: $ERROR_CODE\n기존 이슈: #$EXISTING_ISSUE\n\n근본 원인 해결이 필요합니다."
    fi

    exit 0
fi

#######################################
# 우선순위 라벨 결정
#######################################
case "$SEVERITY" in
    critical) PRIORITY_LABEL="priority:critical" ;;
    high)     PRIORITY_LABEL="priority:high" ;;
    medium)   PRIORITY_LABEL="priority:medium" ;;
    *)        PRIORITY_LABEL="priority:low" ;;
esac

# 도메인 라벨 결정 (에러 코드 접두사 기반)
DOMAIN_LABEL=""
case "$ERROR_CODE" in
    AI*)   DOMAIN_LABEL="domain:ai" ;;
    AUTH*) DOMAIN_LABEL="domain:auth" ;;
    DB*)   DOMAIN_LABEL="domain:db" ;;
esac

# 라벨 조합
LABELS="bug,ai-generated,$PRIORITY_LABEL"
if [ -n "$DOMAIN_LABEL" ]; then
    LABELS="$LABELS,$DOMAIN_LABEL"
fi

#######################################
# 새 Issue 생성
#######################################
echo "새 이슈 생성 중..."

# Auto-Fix 가능 여부 표시
if [ "$AUTO_FIXABLE" = "true" ]; then
    AUTO_FIX_STATUS="> 이 에러는 **자동 수정 가능**으로 분류되었습니다. Phase 5 구현 후 자동 PR이 생성됩니다."
else
    AUTO_FIX_STATUS="> 이 에러는 **수동 수정 필요**로 분류되었습니다."
fi

ISSUE_URL=$(gh issue create \
    --title "[$ERROR_CODE] $ROOT_CAUSE" \
    --label "$LABELS" \
    --body "$(cat <<EOF
## AI 자동 생성 이슈

$AUTO_FIX_STATUS

### 에러 정보

| 항목 | 내용 |
|------|------|
| 에러 코드 | \`$ERROR_CODE\` |
| 심각도 | **$SEVERITY** |
| 자동 수정 | $AUTO_FIXABLE |
| 발생 시간 | $(date -u +"%Y-%m-%dT%H:%M:%SZ") |

### 근본 원인 분석

$ROOT_CAUSE

### 영향 범위

$IMPACT

### 권장 조치

$RECOMMENDATIONS

### 관련 로그

\`\`\`
에러 코드: $ERROR_CODE
발생 시간: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
\`\`\`

---

<details>
<summary>진단 원본 데이터</summary>

\`\`\`json
$DIAGNOSTIC_JSON
\`\`\`

</details>

---
*이 이슈는 AI Monitor에 의해 자동 생성되었습니다.*
EOF
)")

echo "이슈 생성 완료: $ISSUE_URL"

#######################################
# Discord 알림
#######################################
if [ -f "./scripts/discord-alert.sh" ]; then
    ./scripts/discord-alert.sh "$SEVERITY" "새 이슈 생성" \
        "에러 코드: $ERROR_CODE\n이슈: $ISSUE_URL\n\n$ROOT_CAUSE"
fi

echo "=== Issue 생성 프로세스 완료 ==="
```

### 3.3 Log Watcher 연동 (수정)

**파일**: `scripts/log-watcher.sh` (Phase 4 연동 부분 추가)

```bash
# ... 기존 코드 ...

# Phase 3: AI 진단 호출
DIAGNOSTIC_RESULT=$(python3 ./scripts/diagnostic-agent.py "$ERROR_LOG")

# Phase 4: Issue 자동 생성 (신규 추가)
if [ -n "$DIAGNOSTIC_RESULT" ]; then
    ./scripts/create-issue.sh "$DIAGNOSTIC_RESULT"
fi

# ... 기존 코드 ...
```

---

## 4. Issue 템플릿

### 4.1 자동 생성 Issue 형식

```markdown
## AI 자동 생성 이슈

> 이 에러는 **자동 수정 가능**으로 분류되었습니다.

### 에러 정보

| 항목 | 내용 |
|------|------|
| 에러 코드 | `AI5001` |
| 심각도 | **critical** |
| 자동 수정 | true |
| 발생 시간 | 2026-02-06T10:30:00Z |

### 근본 원인 분석

Claude API 호출 시 타임아웃 발생. 네트워크 지연 또는 API 서버 과부하 추정.

### 영향 범위

AI 기반 회고 생성 기능 전체 영향. 사용자 요청 실패.

### 권장 조치

- 타임아웃 값 30초 → 45초로 조정
- 재시도 로직 추가 검토
- API 상태 모니터링 강화
```

### 4.2 재발생 코멘트 형식

```markdown
## 에러 재발생 감지

| 항목 | 내용 |
|------|------|
| 발생 시간 | 2026-02-06T14:20:00Z |
| 에러 코드 | `AI5001` |
| 심각도 | critical |

### 근본 원인 (재분석)
동일한 타임아웃 에러 재발생. 이전 조치가 효과적이지 않음.

### 영향 범위
AI 기능 전체 영향 지속.

---
*AI Monitor 자동 코멘트*
```

---

## 5. 라벨 체계

### 5.1 우선순위 라벨

| 라벨 | 색상 | 설명 | 대응 시간 |
|------|------|------|----------|
| `priority:critical` | 빨강 (#b60205) | 즉시 대응 | < 1시간 |
| `priority:high` | 주황 (#d93f0b) | 우선 대응 | < 4시간 |
| `priority:medium` | 노랑 (#fbca04) | 일반 | < 1일 |
| `priority:low` | 초록 (#0e8a16) | 낮음 | < 1주 |

### 5.2 도메인 라벨

| 라벨 | 색상 | 에러 코드 접두사 |
|------|------|-----------------|
| `domain:ai` | 파랑 (#1d76db) | AI* |
| `domain:auth` | 보라 (#5319e7) | AUTH* |
| `domain:db` | 진파랑 (#0052cc) | DB* |

### 5.3 자동화 라벨

| 라벨 | 설명 |
|------|------|
| `ai-generated` | AI가 자동 생성한 이슈 |
| `auto-fix` | 자동 수정 시도됨 (Phase 5) |

---

## 6. 환경 설정

### 6.1 필수 환경 변수

```bash
# GitHub CLI 인증 (gh auth login으로 설정)
# 또는 환경 변수로 설정
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
```

### 6.2 사전 준비

```bash
# 1. gh CLI 설치
brew install gh  # macOS
# 또는
sudo apt install gh  # Ubuntu

# 2. GitHub 인증
gh auth login

# 3. 라벨 생성 (최초 1회)
chmod +x scripts/setup-labels.sh
./scripts/setup-labels.sh

# 4. 스크립트 실행 권한
chmod +x scripts/create-issue.sh
```

---

## 7. 테스트 시나리오

### 7.1 신규 Issue 생성 테스트

**입력:**
```bash
./scripts/create-issue.sh '{
    "error_code": "AI5001",
    "severity": "critical",
    "root_cause": "Claude API 타임아웃",
    "impact": "AI 기능 전체 영향",
    "recommendations": [
        {"effort": "low", "action": "타임아웃 값 조정"},
        {"effort": "medium", "action": "재시도 로직 추가"}
    ],
    "auto_fixable": true
}'
```

**예상 결과:**
- GitHub Issue 생성
- 라벨: `bug`, `ai-generated`, `priority:critical`, `domain:ai`
- Discord 알림 발송

### 7.2 중복 Issue 테스트

**시나리오:** 동일 에러코드로 열린 이슈 존재

**예상 결과:**
- 새 Issue 생성 안 함
- 기존 이슈에 코멘트 추가
- Discord 알림 ("에러 재발생")

### 7.3 낮은 심각도 테스트

**입력:**
```bash
./scripts/create-issue.sh '{
    "error_code": "COMMON400",
    "severity": "low",
    "root_cause": "입력 검증 실패"
}'
```

**예상 결과:**
- Issue 생성 안 함 (medium/low는 대상 아님)
- 로그만 기록

---

## 8. 구현 체크리스트

### Day 1-2: 기본 구현

- [ ] `scripts/setup-labels.sh` 작성
- [ ] `scripts/create-issue.sh` 작성
- [ ] gh CLI 설치 및 인증 테스트
- [ ] 라벨 생성 실행

### Day 3-4: 연동 및 테스트

- [ ] `log-watcher.sh`에 Phase 4 연동 추가
- [ ] 신규 Issue 생성 테스트
- [ ] 중복 감지 테스트
- [ ] Discord 알림 연동 테스트

### Day 5: 검증 및 문서화

- [ ] E2E 테스트 (실제 에러 시뮬레이션)
- [ ] 에러 케이스 처리 확인
- [ ] 운영 가이드 문서화

---

## 9. 에러 처리

### 9.1 gh CLI 미설치

```bash
if ! command -v gh &> /dev/null; then
    echo "ERROR: gh CLI가 설치되지 않았습니다."
    echo "설치: brew install gh (macOS) 또는 sudo apt install gh (Ubuntu)"
    exit 1
fi
```

### 9.2 인증 실패

```bash
if ! gh auth status &> /dev/null; then
    echo "ERROR: GitHub 인증이 필요합니다."
    echo "실행: gh auth login"
    exit 1
fi
```

### 9.3 JSON 파싱 실패

```bash
if ! echo "$DIAGNOSTIC_JSON" | jq empty 2>/dev/null; then
    echo "ERROR: 유효하지 않은 JSON 입력"
    exit 1
fi
```

---

## 10. 다음 단계

Phase 4 완료 후 Phase 5 (Auto-Fix & PR) 진행:

- [ ] Claude Code CLI 연동
- [ ] 자동 코드 수정
- [ ] Draft PR 생성
- [ ] 테스트 검증 및 롤백

---

## 참고 문서

- [Phase 1: Event Trigger](./phase-1-event-trigger.md)
- [Phase 2: Issue Analysis](./phase-2-issue-analysis.md)
- [Phase 3: AI Diagnostic](./phase-3-ai-diagnostic.md)
- [Phase 5: Auto-Fix & PR](./phase-5-auto-fix-pr.md)
- [Overview](./overview.md)

---

#github-issue #automation #phase4 #ai-monitoring
