# Phase 5: Auto-Fix & PR Automation

> **버전**: 1.1
> **최종 수정**: 2026-02-06
> **상태**: 구현 완료
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
├── log-watcher.sh          # Phase 2 (기존, Phase 5 연동 추가)
├── discord-alert.sh        # Phase 2 (기존)
├── diagnostic-agent.py     # Phase 3 (기존)
├── create-issue.sh         # Phase 4 (기존)
├── setup-labels.sh         # Phase 4 (기존, auto-fix 라벨 포함)
├── auto-fix.sh             # Phase 5 (구현 완료) ✅
├── create-pr.sh            # Phase 5 (구현 완료) ✅
├── rollback-fix.sh         # Phase 5 (구현 완료) ✅
└── verify-fix.sh           # Phase 5 (구현 완료) ✅
```

---

## 3. 구현 상세

### 3.1 Auto-Fix Agent (구현 완료)

**파일**: `scripts/auto-fix.sh`

**주요 기능:**
- Claude Code CLI 연동 (`claude --dangerously-skip-permissions -p`)
- Rate Limiting (일일 5개 PR 제한)
- 수정 범위 검증 (`validate_fix_scope`)
- 롤백 및 정리 로직
- Draft PR 자동 생성
- Discord 알림

**핵심 로직:**

```bash
# 주요 함수들

# 수정 범위 검증 - 허용/금지 키워드 체크
validate_fix_scope() {
    local fix_suggestion="$1"
    local fix_lower=$(echo "$fix_suggestion" | tr '[:upper:]' '[:lower:]')

    # 불허 키워드 (아키텍처, 비즈니스 로직, 보안 등)
    local forbidden_patterns=(
        "architecture" "아키텍처" "business logic" "비즈니스 로직"
        "security" "보안" "authentication" "인증" "authorization" "권한"
        "database schema" "데이터베이스 스키마" "migration" "마이그레이션"
    )

    # 허용 키워드 (타임아웃, 재시도, 로깅, 오타 등)
    local allowed_patterns=(
        "timeout" "타임아웃" "retry" "재시도" "log" "로그"
        "null" "check" "체크" "config" "설정" "patch" "패치"
    )
    # ...
}

# Claude Code CLI 호출
apply_fix() {
    local fix_suggestion="$1"
    local target="${2:-codes/server/src}"
    local error_code="${3:-UNKNOWN}"

    # --dangerously-skip-permissions: CI/CD 자동화 환경용
    claude --dangerously-skip-permissions -p "$prompt"
}

# 테스트 실행
run_tests() {
    cargo test --manifest-path "$SERVER_DIR/Cargo.toml"
    cargo clippy --manifest-path "$SERVER_DIR/Cargo.toml" -- -D warnings
}
```

**실행 흐름:**
1. JSON 입력 검증 및 `auto_fixable` 체크
2. Rate Limit 확인 (일일 5개)
3. 수정 범위 검증 (`validate_fix_scope`)
4. Worktree 상태 확인 (미커밋 변경 방지)
5. 브랜치 생성 (`fix/auto-{ERROR_CODE}-{TIMESTAMP}`)
6. Claude Code CLI로 수정 적용
7. 테스트 실행 (cargo test + clippy)
8. 커밋 및 푸시
9. Draft PR 생성
10. Discord 알림

### 3.2 범용 PR 생성 스크립트 (구현 완료)

**파일**: `scripts/create-pr.sh`

**주요 기능:**
- 범용 PR 생성 (Auto-Fix 외 일반 용도로도 사용 가능)
- PR 타입 지원 (FEAT, FIX, REFACTOR, DOCS, TEST)
- 자동 라벨 할당 (경로 기반 감지)
- 변경사항 요약 자동 생성
- Draft PR 옵션

**사용법:**

```bash
# 기능 추가 PR 생성
./scripts/create-pr.sh -t FEAT -T "회고 좋아요 토글 API 구현"

# 버그 수정 Draft PR 생성
./scripts/create-pr.sh -t FIX -T "회고 삭제 시 500 에러 수정" --draft

# 커스텀 본문과 리뷰어 지정
./scripts/create-pr.sh -t REFACTOR -T "AI 서비스 모듈 분리" -B pr-body.md -r "team-lead"
```

**핵심 기능:**

```bash
# 변경된 파일 기반 라벨 자동 할당
get_auto_labels() {
    local labels=("ai-generated")

    if echo "$changed_files" | grep -q "^codes/server/src/domain/"; then
        labels+=("domain")
    fi
    if echo "$changed_files" | grep -q "^codes/server/tests/"; then
        labels+=("test")
    fi
    if echo "$changed_files" | grep -q "^docs/"; then
        labels+=("documentation")
    fi
    # ...
}

# 변경사항 요약 자동 생성
generate_changes_summary() {
    # Added, Modified, Deleted 섹션 자동 생성
    # 커밋 메시지 요약 포함
}
```

### 3.3 롤백 스크립트 (구현 완료)

**파일**: `scripts/rollback-fix.sh`

**사용법:**
```bash
./scripts/rollback-fix.sh <branch-name>
# 예시: ./scripts/rollback-fix.sh fix/auto-AI5001-20260206143025
```

**기능:**
- 로컬 브랜치 삭제
- 원격 브랜치 삭제
- 관련 PR 자동 닫기 (gh CLI 사용)

### 3.4 검증 스크립트 (구현 완료)

**파일**: `scripts/verify-fix.sh`

**5단계 검증:**
1. 문법 검증 (`cargo check`)
2. 컴파일 검증 (`cargo build`)
3. 테스트 검증 (`cargo test`)
4. Clippy 검증 (`cargo clippy -- -D warnings`)
5. 포맷팅 검증 (`cargo fmt --check`)

### 3.5 log-watcher.sh Phase 5 연동 (구현 완료)

**위치**: `scripts/log-watcher.sh` (Line 277-281)

```bash
# Auto-Fix 시도 (auto_fixable인 경우만)
if [ "$AUTO_FIXABLE" = "true" ]; then
    echo "[$(date)] Attempting Auto-Fix for: $ERROR_CODE"
    "$SCRIPT_DIR/auto-fix.sh" "$DIAGNOSTIC_WITH_CODE" || true
fi
```

**동작 흐름:**
1. 진단 결과에서 `auto_fixable` 필드 확인
2. `true`인 경우 `auto-fix.sh` 자동 호출
3. 실패해도 전체 파이프라인은 계속 진행 (`|| true`)

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

### 5.6 크로스 플랫폼 지원 (macOS/Linux)

구현된 스크립트들은 macOS와 Linux 모두에서 동작합니다:

```bash
# SHA256 해시 (중복 방지용)
sha256_hash() {
    if command -v sha256sum &>/dev/null; then
        sha256sum | cut -d' ' -f1      # Linux
    elif command -v shasum &>/dev/null; then
        shasum -a 256 | cut -d' ' -f1  # macOS
    elif command -v openssl &>/dev/null; then
        openssl dgst -sha256 | awk '{print $NF}'  # fallback
    fi
}

# 락 획득 (동시 실행 방지)
acquire_lock() {
    if command -v flock &>/dev/null; then
        flock -w "$timeout" 200        # Linux
    else
        # macOS fallback: mkdir은 atomic operation
        while ! mkdir "$lock_dir" 2>/dev/null; do
            sleep 1
        done
    fi
}
```

### 5.7 Worktree 상태 확인

```bash
# 미커밋 변경이 새 브랜치에 섞이는 것 방지
if ! git diff --quiet || ! git diff --cached --quiet; then
    log_error "Working tree is not clean"
    exit 1
fi
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

# 3. 라벨 설정 (auto-fix 라벨 포함)
# setup-labels.sh가 이미 auto-fix 라벨을 포함하고 있음
./scripts/setup-labels.sh

# 4. 스크립트 실행 권한
chmod +x scripts/auto-fix.sh
chmod +x scripts/create-pr.sh
chmod +x scripts/rollback-fix.sh
chmod +x scripts/verify-fix.sh
```

**setup-labels.sh에 포함된 라벨:**
- `ai-generated` - AI 모니터링 시스템이 자동 생성
- `priority:critical/high/medium/low` - 우선순위
- `domain:ai/auth/db` - 도메인
- `auto-fix` - AI 자동 수정 PR (Phase 5)

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

- [x] Claude Code CLI 설치 및 테스트
- [x] `scripts/auto-fix.sh` 구현
- [x] `scripts/create-pr.sh` 구현
- [x] Rate Limiting 구현 (일일 5개 PR 제한)
- [x] 롤백 로직 구현 (`scripts/rollback-fix.sh`)

### Week 2: 연동 및 테스트

- [x] `log-watcher.sh`에 Phase 5 연동 (auto_fixable 체크 후 auto-fix.sh 호출)
- [x] 성공 케이스 테스트
- [x] 실패 케이스 테스트 (롤백 확인)
- [x] Rate Limit 테스트
- [x] 금지 파일 보호 테스트
- [x] Issue-PR 연결 테스트

### 추가 구현

- [x] `scripts/verify-fix.sh` 구현 (5단계 검증: syntax, compile, test, clippy, format)
- [x] `scripts/setup-labels.sh`에 `auto-fix` 라벨 추가
- [x] 크로스 플랫폼 지원 (macOS/Linux 호환 - sha256, flock fallback)
- [x] 수정 범위 검증 로직 (`validate_fix_scope`)
- [x] Worktree 상태 확인 (미커밋 변경 방지)

### 검증

- [x] E2E 테스트 (실제 에러 시뮬레이션)
- [x] Draft PR 생성 확인
- [x] Discord 알림 모든 시나리오
- [x] 롤백 완전성 검증

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
