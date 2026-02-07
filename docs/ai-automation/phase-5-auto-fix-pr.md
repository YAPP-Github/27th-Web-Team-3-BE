# Phase 5: Auto-Fix & PR Automation

> **버전**: 1.0
> **최종 수정**: 2026-02-06
> **상태**: 구현 대기
> **의존성**: Phase 4 완료 필수

---

## 개요

| 항목 | 내용 |
|------|------|
| Phase | 5: Auto-Fix & PR Automation |
| 기간 | 1-2주 |
| 목표 | AI 진단 결과 기반 자동 코드 수정 및 Draft PR 생성 |
| 선행 조건 | Phase 4 (Issue Automation) 완료 |
| 복잡도 | 높음 |

---

## 1. 목표 및 범위

### 1.1 왜 필요한가?

Phase 4에서 Issue가 자동 생성되지만, 실제 코드 수정은 여전히 개발자가 수동으로 수행해야 합니다.

**현재 문제:**
- 간단한 수정도 개발자 개입 필요
- 야간/주말 장애 대응 지연
- 반복적인 수정 작업 부담

**Phase 5 해결:**
- `auto_fixable: true` 에러 자동 수정
- Claude Code CLI로 코드 수정 적용
- Draft PR 생성으로 리뷰 프로세스 연계
- 테스트 실패 시 안전한 롤백

### 1.2 목표

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Phase 5 목표                                      │
├─────────────────────────────────────────────────────────────────────────┤
│  1. auto_fixable 에러 자동 코드 수정                                     │
│  2. 테스트 검증 (cargo test + clippy)                                   │
│  3. Draft PR 자동 생성                                                   │
│  4. 실패 시 롤백 + Issue 생성                                            │
│  5. Rate Limiting으로 비용 제어                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.3 범위

**포함 (In Scope):**
- Auto-Fix Agent 구현
- Claude Code CLI 연동
- Draft PR 자동 생성
- 테스트 기반 검증
- 롤백 메커니즘
- Rate Limiting

**제외 (Out of Scope):**
- 자동 머지 (모든 PR은 사람이 검토)
- 비즈니스 로직 수정
- 보안 코드 수정
- 대규모 리팩토링

---

## 2. 아키텍처

### 2.1 전체 흐름

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Phase 5 Auto-Fix 파이프라인                          │
└─────────────────────────────────────────────────────────────────────────┘

  Phase 4 완료 (Issue 생성됨)
       │
       ▼
┌──────────────────┐
│ auto_fixable?    │
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
  true       false
    │         │
    │         ▼
    │      ┌───────────┐
    │      │ 종료      │
    │      │ (Issue만) │
    │      └───────────┘
    ▼
┌──────────────────┐
│ Rate Limit 체크  │
│ (일일 5개 PR)    │
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
  OK       초과
    │         │
    │         ▼
    │      ┌───────────┐
    │      │ 다음날    │
    │      │ 재시도    │
    │      └───────────┘
    ▼
┌──────────────────┐
│ 1. 브랜치 생성   │
│ fix/auto-{code}  │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ 2. Claude Code   │
│    수정 적용     │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ 3. 테스트 실행   │
│ cargo test       │
│ cargo clippy     │
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
  통과       실패
    │         │
    ▼         ▼
┌───────┐   ┌───────────┐
│ 4.커밋│   │ 롤백      │
│ 5.푸시│   │ 브랜치삭제│
│ 6.PR  │   │ Issue생성 │
└───┬───┘   └───────────┘
    │
    ▼
┌───────────┐
│ Draft PR  │
│ 생성 완료 │
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
├── create-issue.sh         # Phase 4 (기존)
├── setup-labels.sh         # Phase 4 (기존)
├── auto-fix.sh             # Phase 5 (신규) ⬅️
└── create-pr.sh            # Phase 5 (신규) ⬅️
```

---

## 3. 구현 상세

### 3.1 Auto-Fix Agent

**파일**: `scripts/auto-fix.sh`

```bash
#!/bin/bash
set -euo pipefail

#######################################
# Auto-Fix Agent
# Phase 5: Auto-Fix & PR Automation
#######################################

# 입력 검증
if [ -z "${1:-}" ]; then
    echo "Usage: $0 <diagnostic_json>"
    exit 1
fi

DIAGNOSTIC_JSON="$1"

# JSON 파싱
ERROR_CODE=$(echo "$DIAGNOSTIC_JSON" | jq -r '.error_code // "UNKNOWN"')
FIX_SUGGESTION=$(echo "$DIAGNOSTIC_JSON" | jq -r '.fix_suggestion // ""')
AUTO_FIXABLE=$(echo "$DIAGNOSTIC_JSON" | jq -r '.auto_fixable // false')
ROOT_CAUSE=$(echo "$DIAGNOSTIC_JSON" | jq -r '.root_cause // ""')

echo "=== Auto-Fix 프로세스 시작 ==="
echo "에러 코드: $ERROR_CODE"
echo "자동 수정 가능: $AUTO_FIXABLE"

#######################################
# 사전 검증
#######################################

# auto_fixable 체크
if [ "$AUTO_FIXABLE" != "true" ]; then
    echo "자동 수정 불가능한 에러입니다."
    exit 0
fi

# fix_suggestion 체크
if [ -z "$FIX_SUGGESTION" ]; then
    echo "수정 제안이 없습니다. Issue만 생성합니다."
    ./scripts/create-issue.sh "$DIAGNOSTIC_JSON"
    exit 0
fi

#######################################
# Rate Limiting (일일 5개 PR)
#######################################
RATE_LIMIT_FILE="/tmp/auto-fix-rate-limit"
TODAY=$(date +%Y-%m-%d)

check_rate_limit() {
    if [ -f "$RATE_LIMIT_FILE" ]; then
        LAST_DATE=$(head -1 "$RATE_LIMIT_FILE")
        if [ "$LAST_DATE" = "$TODAY" ]; then
            COUNT=$(tail -1 "$RATE_LIMIT_FILE")
            if [ "$COUNT" -ge 5 ]; then
                echo "일일 PR 생성 제한 (5개) 도달"
                return 1
            fi
        else
            # 새로운 날짜, 리셋
            echo "$TODAY" > "$RATE_LIMIT_FILE"
            echo "0" >> "$RATE_LIMIT_FILE"
        fi
    else
        echo "$TODAY" > "$RATE_LIMIT_FILE"
        echo "0" >> "$RATE_LIMIT_FILE"
    fi
    return 0
}

increment_rate_limit() {
    COUNT=$(tail -1 "$RATE_LIMIT_FILE")
    NEW_COUNT=$((COUNT + 1))
    echo "$TODAY" > "$RATE_LIMIT_FILE"
    echo "$NEW_COUNT" >> "$RATE_LIMIT_FILE"
    echo "오늘 생성된 PR: $NEW_COUNT/5"
}

if ! check_rate_limit; then
    echo "Rate limit 도달. 내일 다시 시도합니다."
    echo "Issue만 생성합니다."
    ./scripts/create-issue.sh "$DIAGNOSTIC_JSON"
    exit 0
fi

#######################################
# 브랜치 생성
#######################################
TIMESTAMP=$(date +%Y%m%d%H%M%S)
BRANCH="fix/auto-${ERROR_CODE}-${TIMESTAMP}"
ORIGINAL_BRANCH=$(git branch --show-current)

echo "브랜치 생성: $BRANCH"
git checkout -b "$BRANCH"

# 롤백 함수
rollback() {
    echo "롤백 수행 중..."
    git checkout "$ORIGINAL_BRANCH"
    git branch -D "$BRANCH" 2>/dev/null || true
    echo "롤백 완료"
}

# 에러 시 롤백
trap 'rollback' ERR

#######################################
# Claude Code CLI로 수정 적용
#######################################
echo "Claude Code로 수정 적용 중..."

# Claude Code CLI 실행
# --print 옵션으로 출력만 수행
claude --print "
당신은 Rust 백엔드 코드를 수정하는 AI입니다.
다음 규칙을 반드시 따르세요:

1. 최소한의 변경만 수행하세요
2. 기존 코드 스타일을 유지하세요
3. 테스트 코드는 수정하지 마세요
4. 다음 파일은 절대 수정하지 마세요:
   - main.rs
   - config/
   - migrations/
   - .env*
   - *.sql
   - Cargo.toml

수정 내용:
$FIX_SUGGESTION

에러 코드: $ERROR_CODE
근본 원인: $ROOT_CAUSE
"

#######################################
# 변경사항 확인
#######################################
if [ -z "$(git status --porcelain)" ]; then
    echo "변경사항 없음. 롤백합니다."
    rollback
    echo "Issue만 생성합니다."
    ./scripts/create-issue.sh "$DIAGNOSTIC_JSON"
    exit 0
fi

echo "변경된 파일:"
git status --short

#######################################
# 금지 파일 변경 체크
#######################################
FORBIDDEN_PATTERNS="main.rs|config/|migrations/|\.env|\.sql|Cargo.toml"
CHANGED_FILES=$(git status --porcelain | awk '{print $2}')

for file in $CHANGED_FILES; do
    if echo "$file" | grep -qE "$FORBIDDEN_PATTERNS"; then
        echo "ERROR: 금지된 파일이 변경되었습니다: $file"
        rollback
        ./scripts/create-issue.sh "$DIAGNOSTIC_JSON"
        exit 1
    fi
done

#######################################
# 테스트 실행
#######################################
echo "테스트 실행 중..."
cd codes/server

# cargo test
echo "1/2: cargo test..."
if ! cargo test 2>&1; then
    echo "테스트 실패!"
    cd ../..
    rollback

    # 실패 정보 추가하여 Issue 생성
    FAIL_JSON=$(echo "$DIAGNOSTIC_JSON" | jq '. + {"auto_fix_failed": true, "fail_reason": "cargo test failed"}')
    ./scripts/create-issue.sh "$FAIL_JSON"

    # Discord 알림
    ./scripts/discord-alert.sh "warning" "Auto-Fix 실패" \
        "에러 코드: $ERROR_CODE\n실패 원인: 테스트 실패\n\n수동 수정이 필요합니다."
    exit 1
fi

# cargo clippy
echo "2/2: cargo clippy..."
if ! cargo clippy -- -D warnings 2>&1; then
    echo "Clippy 검사 실패!"
    cd ../..
    rollback

    FAIL_JSON=$(echo "$DIAGNOSTIC_JSON" | jq '. + {"auto_fix_failed": true, "fail_reason": "cargo clippy failed"}')
    ./scripts/create-issue.sh "$FAIL_JSON"

    ./scripts/discord-alert.sh "warning" "Auto-Fix 실패" \
        "에러 코드: $ERROR_CODE\n실패 원인: Clippy 검사 실패\n\n수동 수정이 필요합니다."
    exit 1
fi

cd ../..
echo "모든 테스트 통과!"

#######################################
# 커밋 및 푸시
#######################################
echo "변경사항 커밋 중..."

git add -A
git commit -m "$(cat <<EOF
fix: [$ERROR_CODE] 자동 수정

$FIX_SUGGESTION

근본 원인: $ROOT_CAUSE

Co-Authored-By: AI Monitor <ai-monitor@yapp.co.kr>
EOF
)"

echo "원격 저장소에 푸시 중..."
git push -u origin "$BRANCH"

#######################################
# Draft PR 생성
#######################################
echo "Draft PR 생성 중..."
./scripts/create-pr.sh "$BRANCH" "$DIAGNOSTIC_JSON"

# Rate limit 증가
increment_rate_limit

# 원래 브랜치로 복귀
git checkout "$ORIGINAL_BRANCH"

echo "=== Auto-Fix 프로세스 완료 ==="
```

### 3.2 Draft PR 생성

**파일**: `scripts/create-pr.sh`

```bash
#!/bin/bash
set -euo pipefail

#######################################
# Draft PR 생성 스크립트
# Phase 5: Auto-Fix & PR Automation
#######################################

if [ -z "${1:-}" ] || [ -z "${2:-}" ]; then
    echo "Usage: $0 <branch> <diagnostic_json>"
    exit 1
fi

BRANCH="$1"
DIAGNOSTIC_JSON="$2"

# JSON 파싱
ERROR_CODE=$(echo "$DIAGNOSTIC_JSON" | jq -r '.error_code // "UNKNOWN"')
ROOT_CAUSE=$(echo "$DIAGNOSTIC_JSON" | jq -r '.root_cause // ""')
FIX_SUGGESTION=$(echo "$DIAGNOSTIC_JSON" | jq -r '.fix_suggestion // ""')
IMPACT=$(echo "$DIAGNOSTIC_JSON" | jq -r '.impact // ""')

#######################################
# 변경사항 분석
#######################################
CHANGED_FILES=$(git diff origin/dev..HEAD --name-only | head -10)
DIFF_STAT=$(git diff origin/dev..HEAD --stat | tail -1)

#######################################
# Draft PR 생성
#######################################
echo "Draft PR 생성 중..."

PR_URL=$(gh pr create \
    --draft \
    --title "fix: [$ERROR_CODE] 자동 수정" \
    --label "auto-fix,ai-generated" \
    --base dev \
    --body "$(cat <<EOF
## AI 자동 수정 PR

> **이 PR은 AI에 의해 자동 생성되었습니다. 반드시 리뷰 후 머지하세요.**

### 에러 정보

| 항목 | 내용 |
|------|------|
| 에러 코드 | \`$ERROR_CODE\` |
| 브랜치 | \`$BRANCH\` |
| 생성 시간 | $(date -u +"%Y-%m-%dT%H:%M:%SZ") |

### 근본 원인

$ROOT_CAUSE

### 수정 내용

$FIX_SUGGESTION

### 영향 범위

$IMPACT

### 변경된 파일

\`\`\`
$CHANGED_FILES
\`\`\`

### 변경 통계

\`\`\`
$DIFF_STAT
\`\`\`

### 자동 검증 결과

- [x] \`cargo test\` 통과
- [x] \`cargo clippy -- -D warnings\` 통과
- [x] 금지 파일 변경 없음

### 리뷰 체크리스트

- [ ] 수정 내용이 근본 원인을 해결하는가?
- [ ] 비즈니스 로직에 부정적 영향이 없는가?
- [ ] 추가 테스트 케이스가 필요한가?
- [ ] 성능에 영향이 없는가?

---

<details>
<summary>진단 원본 데이터</summary>

\`\`\`json
$DIAGNOSTIC_JSON
\`\`\`

</details>

---

*이 PR은 AI Monitor에 의해 자동 생성되었습니다.*

Co-Authored-By: AI Monitor <ai-monitor@yapp.co.kr>
EOF
)")

echo "Draft PR 생성 완료: $PR_URL"

#######################################
# Issue와 연결 (있는 경우)
#######################################
# 동일 에러코드로 열린 이슈 찾기
RELATED_ISSUE=$(gh issue list \
    --label "ai-generated" \
    --state open \
    --search "in:title $ERROR_CODE" \
    --json number \
    --jq '.[0].number // empty' 2>/dev/null || echo "")

if [ -n "$RELATED_ISSUE" ]; then
    echo "관련 이슈 #$RELATED_ISSUE 에 PR 연결 코멘트 추가..."
    gh issue comment "$RELATED_ISSUE" --body "$(cat <<EOF
## Auto-Fix PR 생성됨

이 이슈에 대한 자동 수정 PR이 생성되었습니다.

**PR**: $PR_URL

리뷰 후 머지해주세요.

---
*AI Monitor 자동 코멘트*
EOF
)"
fi

#######################################
# Discord 알림
#######################################
if [ -f "./scripts/discord-alert.sh" ]; then
    ./scripts/discord-alert.sh "info" "Auto-Fix PR 생성" \
        "에러 코드: $ERROR_CODE\nPR: $PR_URL\n\n리뷰가 필요합니다."
fi

echo "PR 생성 프로세스 완료"
```

---

## 4. 수정 허용 범위

### 4.1 허용 대상 (Auto-Fix 가능)

| 카테고리 | 예시 | 위험도 |
|----------|------|--------|
| 설정 값 조정 | 타임아웃 30s → 45s | 낮음 |
| 로깅 개선 | 디버그 컨텍스트 추가 | 낮음 |
| Null 체크 | `?.` 연산자 추가 | 낮음 |
| 포맷팅 | clippy 경고 해결 | 낮음 |
| 에러 메시지 | 더 명확한 메시지 | 낮음 |

### 4.2 금지 대상 (수동 수정 필요)

| 카테고리 | 예시 | 이유 |
|----------|------|------|
| 아키텍처 변경 | 모듈 구조 변경 | 영향 불확실 |
| 비즈니스 로직 | 검증 규칙 변경 | 요구사항 확인 필요 |
| 보안 코드 | 인증/인가 로직 | 보안 검토 필수 |
| DB 스키마 | 마이그레이션 | 데이터 손실 위험 |
| API 인터페이스 | 응답 형식 변경 | 클라이언트 영향 |
| 의존성 변경 | Cargo.toml 수정 | 호환성 검토 필요 |

### 4.3 금지 파일 목록

```bash
# 절대 자동 수정 불가
main.rs           # 진입점
config/           # 설정
migrations/       # DB 마이그레이션
.env*             # 환경 변수
*.sql             # SQL 쿼리
Cargo.toml        # 의존성
Cargo.lock        # 의존성 락
```

### 4.4 변경 제한

| 항목 | 제한 |
|------|------|
| 파일 수 | 최대 3개 |
| 파일당 변경 라인 | 최대 50줄 |
| 함수 추가 | 불가 |
| 타입 추가 | 불가 |

---

## 5. 안전 장치

### 5.1 Rate Limiting

| 항목 | 제한 | 이유 |
|------|------|------|
| PR 생성 | 일일 5개 | 리뷰 부담 방지 |
| Claude API | 시간당 10회 | API 비용 제어 |

### 5.2 테스트 필수

```bash
# 모든 테스트 통과 필수
cargo test                    # 단위/통합 테스트
cargo clippy -- -D warnings   # 린트 검사
```

### 5.3 롤백 정책

```
테스트 실패 시:
1. git checkout $ORIGINAL_BRANCH
2. git branch -D $BRANCH
3. Issue 생성 (auto_fix_failed: true)
4. Discord 알림 (실패 알림)
```

### 5.4 Draft PR 강제

- **모든 Auto-Fix PR은 Draft 상태**
- 자동 머지 **절대 불가**
- 리뷰어 승인 필수

### 5.5 금지 파일 보호

```bash
# 변경 전 검증
FORBIDDEN_PATTERNS="main.rs|config/|migrations/|\.env|\.sql|Cargo.toml"

for file in $CHANGED_FILES; do
    if echo "$file" | grep -qE "$FORBIDDEN_PATTERNS"; then
        echo "ERROR: 금지된 파일 변경"
        rollback
        exit 1
    fi
done
```

---

## 6. Claude Code CLI 연동

### 6.1 설치

```bash
# Claude Code CLI 설치
# (공식 문서 참조)
npm install -g @anthropic-ai/claude-code
# 또는
brew install claude-code
```

### 6.2 인증

```bash
# API 키 설정
export ANTHROPIC_API_KEY=sk-ant-xxxxx
```

### 6.3 프롬프트 가이드라인

```
당신은 Rust 백엔드 코드를 수정하는 AI입니다.

규칙:
1. 최소한의 변경만 수행
2. 기존 코드 스타일 유지
3. 테스트 코드 수정 금지
4. 금지 파일 수정 금지:
   - main.rs
   - config/
   - migrations/
   - .env*
   - *.sql
   - Cargo.toml

수정 내용: {fix_suggestion}
에러 코드: {error_code}
```

---

## 7. 환경 설정

### 7.1 필수 환경 변수

```bash
# .env.monitoring (gitignore에 포함)
DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/...
ANTHROPIC_API_KEY=sk-ant-...      # Claude Code CLI
GITHUB_TOKEN=ghp_...               # gh CLI 인증
```

### 7.2 사전 준비

```bash
# 1. Claude Code CLI 설치
npm install -g @anthropic-ai/claude-code

# 2. gh CLI 인증 (Phase 4에서 완료)
gh auth login

# 3. auto-fix 라벨 추가 (최초 1회)
gh label create "auto-fix" \
    --color "0e8a16" \
    --description "AI 자동 수정 PR" \
    --force

# 4. 스크립트 실행 권한
chmod +x scripts/auto-fix.sh
chmod +x scripts/create-pr.sh
```

---

## 8. 테스트 시나리오

### 8.1 Auto-Fix 성공 케이스

**입력:**
```bash
./scripts/auto-fix.sh '{
    "error_code": "AI5001",
    "severity": "high",
    "root_cause": "타임아웃 설정 부족",
    "impact": "간헐적 API 실패",
    "auto_fixable": true,
    "fix_suggestion": "codes/server/src/domain/ai/service.rs의 TIMEOUT_SECS를 30에서 45로 변경"
}'
```

**예상 결과:**
1. 브랜치 생성: `fix/auto-AI5001-20260206...`
2. Claude Code로 수정 적용
3. 테스트 통과
4. Draft PR 생성
5. Discord 알림

### 8.2 테스트 실패 케이스

**시나리오:** 수정 후 테스트 실패

**예상 결과:**
1. 브랜치 생성
2. Claude Code 수정 적용
3. 테스트 **실패**
4. 롤백 (브랜치 삭제)
5. Issue 생성 (`auto_fix_failed: true`)
6. Discord 알림 ("Auto-Fix 실패")

### 8.3 Rate Limit 케이스

**시나리오:** 일일 5개 PR 초과

**예상 결과:**
1. Rate limit 체크 실패
2. Issue만 생성
3. 로그: "내일 다시 시도"

### 8.4 금지 파일 변경 케이스

**시나리오:** Claude가 main.rs 수정 시도

**예상 결과:**
1. 변경 감지
2. 금지 파일 체크 실패
3. 롤백
4. Issue 생성

---

## 9. 구현 체크리스트

### Week 1: 기본 구현

- [ ] Claude Code CLI 설치 및 테스트
- [ ] `scripts/auto-fix.sh` 구현
- [ ] `scripts/create-pr.sh` 구현
- [ ] Rate Limiting 구현
- [ ] 롤백 로직 구현

### Week 2: 연동 및 테스트

- [ ] `log-watcher.sh`에 Phase 5 연동
- [ ] 성공 케이스 테스트
- [ ] 실패 케이스 테스트 (롤백 확인)
- [ ] Rate Limit 테스트
- [ ] 금지 파일 보호 테스트
- [ ] Issue-PR 연결 테스트

### 검증

- [ ] E2E 테스트 (실제 에러 시뮬레이션)
- [ ] Draft PR 생성 확인
- [ ] Discord 알림 모든 시나리오
- [ ] 롤백 완전성 검증

---

## 10. 비용 예상

| 항목 | 월간 예상 비용 |
|------|--------------|
| Claude API (Auto-Fix) | ~$10-20 |
| GitHub Actions | 무료 (로컬 실행) |
| Discord Webhook | 무료 |
| **합계** | **~$10-20/월** |

---

## 11. 모니터링 지표

### 11.1 성공률

```
Auto-Fix 성공률 = 성공한 PR 수 / 시도한 수정 수 × 100
목표: > 50%
```

### 11.2 추적 지표

| 지표 | 설명 |
|------|------|
| 일일 Auto-Fix 시도 | 시도된 자동 수정 수 |
| 일일 PR 생성 | 성공적으로 생성된 PR 수 |
| 테스트 실패율 | 테스트 실패로 롤백된 비율 |
| 머지된 PR | 사람이 승인하여 머지된 PR 수 |

---

## 12. 다음 단계 (Phase 6 고려사항)

- [ ] 리뷰어 자동 할당
- [ ] 머지 후 배포 자동화
- [ ] Auto-Fix 성공률 대시보드
- [ ] 학습 기반 수정 개선

---

## 참고 문서

- [Phase 1: Event Trigger](./phase-1-event-trigger.md)
- [Phase 2: Issue Analysis](./phase-2-issue-analysis.md)
- [Phase 3: AI Diagnostic](./phase-3-ai-diagnostic.md)
- [Phase 4: Issue Automation](./phase-4-issue-automation.md)
- [Overview](./overview.md)
- [Security & Governance](./security-governance.md)

---

#auto-fix #draft-pr #claude-code #phase5 #ai-monitoring
