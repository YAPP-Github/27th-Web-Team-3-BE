# Phase 4 (Production): 자동화 확장

## 개요

| 항목 | 내용 |
|------|------|
| Phase | 4: Production |
| 기간 | Week 7-8 |
| 목표 | GitHub Issue 자동 생성, Auto-Fix PR, 대시보드 |
| 의존성 | Phase 3 (AI) 완료 |

```
Phase 4 완료 상태
┌─────────────────────────────────────────────────────────────┐
│  ✅ GitHub Issue    ✅ Auto-Fix PR    ⬜ 대시보드 (선택)   │
└─────────────────────────────────────────────────────────────┘
```

## 완료 조건

- [ ] 에러 발생 시 GitHub Issue 자동 생성
- [ ] `auto_fixable: true`인 경우 Draft PR 생성
- [ ] 테스트 실패 시 자동 롤백

---

## 사전 조건

### GitHub CLI 설정
```bash
# 설치
brew install gh

# 인증
gh auth login
```

### 환경 변수
```bash
# .env에 추가
GITHUB_TOKEN=ghp_xxx
```

---

## 태스크 4.1: GitHub Issue 자동 생성

### 구현

**파일**: `scripts/create-issue.sh`

```bash
#!/bin/bash
# scripts/create-issue.sh - GitHub Issue 자동 생성

set -e

DIAGNOSTIC="$1"

# 진단 결과 파싱
ERROR_CODE=$(echo "$DIAGNOSTIC" | jq -r '.error_code // "UNKNOWN"')
SEVERITY=$(echo "$DIAGNOSTIC" | jq -r '.severity // "warning"')
ROOT_CAUSE=$(echo "$DIAGNOSTIC" | jq -r '.root_cause // "분석 필요"')
IMPACT=$(echo "$DIAGNOSTIC" | jq -r '.impact // "확인 필요"')
RECOMMENDATIONS=$(echo "$DIAGNOSTIC" | jq -r '[.recommendations[] | "- [\(.effort)] \(.action)"] | join("\n")' 2>/dev/null || echo "- 검토 필요")

# 라벨 설정
case "$SEVERITY" in
    critical) PRIORITY_LABEL="priority:critical" ;;
    warning)  PRIORITY_LABEL="priority:high" ;;
    *)        PRIORITY_LABEL="priority:medium" ;;
esac

# 중복 이슈 체크
EXISTING=$(gh issue list \
    --label "ai-generated" \
    --search "$ERROR_CODE in:title" \
    --state open \
    --json number \
    --jq '.[0].number' 2>/dev/null || echo "")

if [ -n "$EXISTING" ] && [ "$EXISTING" != "null" ]; then
    echo "Adding comment to existing issue #$EXISTING"
    gh issue comment "$EXISTING" --body "### 추가 발생
**시간**: $(date '+%Y-%m-%d %H:%M:%S')

동일한 에러가 다시 감지되었습니다."
    exit 0
fi

# 새 이슈 생성
ISSUE_URL=$(gh issue create \
    --title "[AI Monitor] $ERROR_CODE: $(echo "$ROOT_CAUSE" | head -c 50)" \
    --body "## AI 자동 생성 이슈

### 심각도
\`$SEVERITY\`

### 근본 원인
$ROOT_CAUSE

### 영향 범위
$IMPACT

### 권장 조치
$RECOMMENDATIONS

---
_이 이슈는 AI 모니터링 시스템에 의해 자동 생성되었습니다._
_검토 후 적절한 조치를 취해주세요._" \
    --label "bug" \
    --label "ai-generated" \
    --label "$PRIORITY_LABEL")

echo "Created issue: $ISSUE_URL"
```

### 체크리스트

- [ ] `gh` CLI 설치 및 인증
- [ ] 스크립트 실행 권한
- [ ] 라벨 사전 생성: `ai-generated`, `priority:critical`, `priority:high`, `priority:medium`

---

## 태스크 4.2: Auto-Fix Agent

### 수정 허용 범위

| 허용 | 예시 |
|------|------|
| 설정 값 조정 | 타임아웃, 재시도 횟수, 버퍼 크기 |
| 로깅 개선 | 추가 컨텍스트 로깅 |
| 간단한 버그 | 오타, 누락된 null 체크 |
| 의존성 업데이트 | 패치 버전 업그레이드 |

| 불허 | 이유 |
|------|------|
| 아키텍처 변경 | 사람의 검토 필수 |
| 비즈니스 로직 | 요구사항 확인 필요 |
| 보안 코드 | 보안 검토 필수 |
| 대규모 리팩토링 | 영향 범위 불확실 |

### 구현

**파일**: `scripts/auto-fix.sh`

```bash
#!/bin/bash
# scripts/auto-fix.sh - 자동 수정 PR 생성

set -e

DIAGNOSTIC="$1"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# auto_fixable 확인
AUTO_FIXABLE=$(echo "$DIAGNOSTIC" | jq -r '.auto_fixable // false')
if [ "$AUTO_FIXABLE" != "true" ]; then
    echo "Not auto-fixable, skipping"
    exit 0
fi

FIX_SUGGESTION=$(echo "$DIAGNOSTIC" | jq -r '.fix_suggestion // ""')
ERROR_CODE=$(echo "$DIAGNOSTIC" | jq -r '.error_code // "UNKNOWN"')
ROOT_CAUSE=$(echo "$DIAGNOSTIC" | jq -r '.root_cause // ""')

if [ -z "$FIX_SUGGESTION" ]; then
    echo "No fix suggestion provided"
    exit 0
fi

BRANCH="fix/auto-${ERROR_CODE}-$(date +%s)"

echo "=== Starting Auto-Fix ==="
echo "Branch: $BRANCH"
echo "Fix: $FIX_SUGGESTION"

# 1. 브랜치 생성
cd "$PROJECT_DIR"
git checkout dev
git pull origin dev
git checkout -b "$BRANCH"

# 2. Claude Code로 수정 적용
echo "$FIX_SUGGESTION" | claude --print "
다음 수정을 코드에 적용해주세요.
수정만 적용하고, 테스트는 실행하지 마세요.

수정 내용:
$FIX_SUGGESTION
"

# 3. 변경 사항 확인
if [ -z "$(git status --porcelain)" ]; then
    echo "No changes made, aborting"
    git checkout dev
    git branch -D "$BRANCH"
    exit 0
fi

# 4. 테스트 실행
echo "=== Running Tests ==="
cd codes/server

if ! cargo test; then
    echo "Tests failed, aborting"
    cd "$PROJECT_DIR"
    git checkout dev
    git branch -D "$BRANCH"
    exit 1
fi

if ! cargo clippy -- -D warnings; then
    echo "Clippy failed, aborting"
    cd "$PROJECT_DIR"
    git checkout dev
    git branch -D "$BRANCH"
    exit 1
fi

# 5. 커밋 및 푸시
cd "$PROJECT_DIR"
git add -A
git commit -m "fix($ERROR_CODE): $(echo "$ROOT_CAUSE" | head -c 50)

Auto-generated fix based on AI diagnostic.

Co-Authored-By: AI Monitor <ai-monitor@example.com>"

git push -u origin "$BRANCH"

# 6. Draft PR 생성
PR_URL=$(gh pr create --draft \
    --title "fix($ERROR_CODE): Auto-fix" \
    --body "## AI 자동 생성 PR

### 진단 결과
**심각도**: $(echo "$DIAGNOSTIC" | jq -r '.severity')

**근본 원인**
$ROOT_CAUSE

**적용된 수정**
$FIX_SUGGESTION

---
⚠️ **주의**: 이 PR은 AI에 의해 자동 생성되었습니다.
**반드시 사람이 검토한 후 머지해주세요.**

Labels: \`auto-fix\`, \`ai-generated\`" \
    --label "auto-fix" \
    --label "ai-generated")

echo "=== PR Created ==="
echo "$PR_URL"

# 7. Discord 알림
./scripts/discord-alert.sh "info" \
    "🤖 Auto-Fix PR 생성" \
    "**에러 코드**: $ERROR_CODE\n**PR**: $PR_URL\n\n검토 후 머지해주세요." \
    "$ERROR_CODE"

# dev로 복귀
git checkout dev
```

### 체크리스트

- [ ] `claude` CLI 설치 (Claude Code)
- [ ] `gh` CLI 인증 완료
- [ ] 스크립트 실행 권한
- [ ] 라벨 생성: `auto-fix`

---

## 태스크 4.3: Log Watcher 최종 연동

**파일**: `scripts/log-watcher.sh` (최종 수정)

```bash
# 진단 완료 후 추가
if [ "$SEVERITY" = "critical" ]; then
    # GitHub Issue 생성
    DIAGNOSTIC_WITH_CODE=$(echo "$DIAGNOSTIC" | jq --arg ec "$ERROR_CODE" '. + {error_code: $ec}')
    ./scripts/create-issue.sh "$DIAGNOSTIC_WITH_CODE"

    # Auto-Fix 시도
    ./scripts/auto-fix.sh "$DIAGNOSTIC_WITH_CODE" || true
fi
```

---

## 안전 장치

### 1. Draft PR만 생성
- 자동 머지 없음
- 사람의 검토 필수

### 2. 테스트 필수
```bash
cargo test && cargo clippy -- -D warnings
# 실패 시 브랜치 삭제, PR 생성 안 함
```

### 3. 수정 범위 제한
- 설정 값, 로깅 등 저위험 변경만
- 비즈니스 로직, 보안 코드 수정 불가

### 4. 호출 제한
- 시간당 최대 10회 진단
- 일일 최대 5개 Auto-Fix PR

---

## 성공 지표

| 지표 | 목표 | 측정 방법 |
|------|------|----------|
| 장애 감지 시간 | < 5분 | 로그 타임스탬프 ~ Discord 알림 시간 |
| 진단 정확도 | > 70% | 사람 검토 후 피드백 |
| Auto-Fix 성공률 | > 50% | 머지된 PR / 생성된 PR |
| 알림 응답 시간 | < 30분 | Discord 알림 ~ 첫 반응 |

---

## 산출물

Phase 4 완료 시:

1. **자동 GitHub Issue**
   - 에러 발생 시 자동 생성
   - 중복 이슈 방지

2. **Auto-Fix PR**
   - 단순 수정 자동 제안
   - 테스트 통과 필수

3. **완전한 파이프라인**
```
에러 발생 → 감지 → AI 진단 → Issue 생성 → Auto-Fix PR → Discord 알림
```

---

## 전체 시스템 완성

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI 자율 모니터링 시스템                        │
│                                                                 │
│  [Rust Server] ──▶ [JSON Logs] ──▶ [Log Watcher] ──▶ [Claude]  │
│                                          │              │       │
│                                          ▼              ▼       │
│                                    [Discord]    [GitHub Issue]  │
│                                          │              │       │
│                                          └──────┬───────┘       │
│                                                 ▼               │
│                                          [Auto-Fix PR]          │
└─────────────────────────────────────────────────────────────────┘
```
