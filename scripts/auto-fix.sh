#!/bin/bash
# scripts/auto-fix.sh - 자동 수정 에이전트
# 진단 결과를 기반으로 자동 수정 브랜치 생성 및 Draft PR 제출

set -euo pipefail

# 스크립트 위치 기반 절대 경로 설정
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SERVER_DIR="$PROJECT_ROOT/codes/server"

# ============== 설정 파일 체크 ==============
check_automation_enabled() {
    local config_file="$PROJECT_ROOT/automation.config.yaml"

    if [ ! -f "$config_file" ]; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARN: Config file not found, automation disabled by default"
        return 1
    fi

    if ! python3 "$SCRIPT_DIR/config-loader.py" --check auto_fix 2>/dev/null; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: Auto-fix is disabled in config"
        return 1
    fi

    return 0
}

# 자동화 활성화 체크 (인자가 있을 때만 - usage 출력은 항상 허용)
if [ $# -ge 1 ] && [ "$1" != "--help" ] && [ "$1" != "-h" ]; then
    if ! check_automation_enabled; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Exiting: auto-fix disabled"
        exit 0
    fi
fi

# 설정
BASE_BRANCH="${BASE_BRANCH:-dev}"
MAX_DAILY_PRS=5
STATE_DIR="${STATE_DIR:-$PROJECT_ROOT/logs/.state}"
DAILY_PR_COUNT_FILE="$STATE_DIR/auto-fix-daily-$(date +%Y-%m-%d)"
LOCK_FILE="$STATE_DIR/auto-fix.lock"
LOCK_TIMEOUT=30

# 상태 디렉토리 생성
mkdir -p "$STATE_DIR"

# 크로스 플랫폼 sha256 함수 (macOS/Linux 호환)
sha256_hash() {
    if command -v sha256sum &>/dev/null; then
        sha256sum | cut -d' ' -f1
    elif command -v shasum &>/dev/null; then
        shasum -a 256 | cut -d' ' -f1
    elif command -v openssl &>/dev/null; then
        openssl dgst -sha256 | awk '{print $NF}'
    else
        # fallback: 입력을 그대로 반환 (중복 방지 약화)
        cat
    fi
}

# 크로스 플랫폼 락 획득 함수 (macOS/Linux 호환)
# macOS에서 flock은 기본 제공되지 않으므로 mkdir 기반 atomic lock 사용
acquire_lock() {
    local lock_file="$1"
    local timeout="${2:-30}"

    if command -v flock &>/dev/null; then
        # Linux: flock 사용
        exec 200>"$lock_file"
        flock -w "$timeout" 200
    else
        # macOS fallback: mkdir은 atomic operation
        local lock_dir="${lock_file}.lock"
        local attempts=0

        while ! mkdir "$lock_dir" 2>/dev/null; do
            ((attempts++))
            if [ "$attempts" -ge "$timeout" ]; then
                return 1
            fi
            sleep 1
        done

        # 종료 시 락 해제
        trap 'rmdir "$lock_dir" 2>/dev/null' EXIT
    fi
}

# 크로스 플랫폼 락 해제 함수
release_lock() {
    local lock_file="$1"
    local lock_dir="${lock_file}.lock"

    # mkdir 기반 락인 경우 디렉토리 삭제
    if [ -d "$lock_dir" ]; then
        rmdir "$lock_dir" 2>/dev/null || true
    fi
}

# 로깅 함수
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" >&2
}

log_warn() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARN: $*"
}

# 브랜치명 sanitize (특수문자, 슬래시, 공백 제거)
sanitize_branch_name() {
    local input="$1"
    # 알파벳, 숫자, 하이픈, 밑줄만 허용, 나머지는 하이픈으로 대체
    echo "$input" | tr -cs 'A-Za-z0-9_-' '-' | sed 's/^-//;s/-$//' | head -c 50
}

# 사용법 출력
usage() {
    cat << EOF
Usage: $0 '<diagnostic_json>'

Example:
  $0 '{"severity":"warning","root_cause":"타임아웃 설정 부족","auto_fixable":true,"fix_suggestion":"timeout을 30초에서 60초로 증가"}'

Required JSON fields:
  - auto_fixable: boolean (true일 때만 수정 진행)
  - fix_suggestion: string (구체적 수정 내용)
  - severity: string (critical|warning|info)
  - root_cause: string (근본 원인)

Optional fields:
  - error_code: string (브랜치명에 사용)
  - target: string (수정 대상 위치)

Environment variables:
  - BASE_BRANCH: 기본 브랜치 (default: dev)
  - DISCORD_WEBHOOK_URL: Discord 알림용 웹훅 URL
EOF
    exit 1
}

# 정리 함수 (실패 시 브랜치 삭제)
cleanup_branch() {
    local branch_name="$1"
    local original_branch="$2"

    log_warn "Cleaning up failed fix branch: $branch_name"

    # 원래 브랜치로 복귀
    git -C "$PROJECT_ROOT" checkout "$original_branch" 2>/dev/null || true

    # 실패한 브랜치 삭제 (로컬)
    git -C "$PROJECT_ROOT" branch -D "$branch_name" 2>/dev/null || true

    # 원격 브랜치 삭제 (푸시된 경우)
    git -C "$PROJECT_ROOT" push origin --delete "$branch_name" 2>/dev/null || true
}

# Discord 알림 전송
send_notification() {
    local severity="$1"
    local title="$2"
    local message="$3"
    local error_code="${4:-AUTO-FIX}"

    if [ -x "$SCRIPT_DIR/discord-alert.sh" ]; then
        "$SCRIPT_DIR/discord-alert.sh" "$severity" "$title" "$message" "$error_code" || true
    else
        log_warn "Discord alert script not found or not executable"
    fi
}

# 일일 PR 제한 확인
check_daily_limit() {
    local count=0

    if [ -f "$DAILY_PR_COUNT_FILE" ]; then
        count=$(cat "$DAILY_PR_COUNT_FILE" 2>/dev/null || echo 0)
    fi

    if [ "$count" -ge "$MAX_DAILY_PRS" ]; then
        log_error "Daily PR limit reached ($MAX_DAILY_PRS PRs)"
        return 1
    fi

    log_info "Daily PR count: $count / $MAX_DAILY_PRS"
    return 0
}

# 일일 PR 카운트 증가
increment_daily_count() {
    local count=0

    if [ -f "$DAILY_PR_COUNT_FILE" ]; then
        count=$(cat "$DAILY_PR_COUNT_FILE" 2>/dev/null || echo 0)
    fi

    echo $((count + 1)) > "$DAILY_PR_COUNT_FILE"
}

# 수정 허용 범위 검증
validate_fix_scope() {
    local fix_suggestion="$1"

    # 소문자로 변환하여 검사
    local fix_lower=$(echo "$fix_suggestion" | tr '[:upper:]' '[:lower:]')

    # 불허 키워드 검사
    local forbidden_patterns=(
        "architecture"
        "아키텍처"
        "business logic"
        "비즈니스 로직"
        "security"
        "보안"
        "authentication"
        "인증"
        "authorization"
        "권한"
        "encryption"
        "암호화"
        "database schema"
        "데이터베이스 스키마"
        "migration"
        "마이그레이션"
    )

    for pattern in "${forbidden_patterns[@]}"; do
        if echo "$fix_lower" | grep -qi "$pattern"; then
            log_error "Fix suggestion contains forbidden scope: $pattern"
            return 1
        fi
    done

    # 허용 키워드 확인 (적어도 하나 포함 필요)
    local allowed_patterns=(
        "timeout"
        "타임아웃"
        "retry"
        "재시도"
        "log"
        "로그"
        "logging"
        "로깅"
        "typo"
        "오타"
        "null"
        "check"
        "체크"
        "config"
        "설정"
        "dependency"
        "의존성"
        "patch"
        "패치"
        "version"
        "버전"
    )

    for pattern in "${allowed_patterns[@]}"; do
        if echo "$fix_lower" | grep -qi "$pattern"; then
            log_info "Fix scope validated: matches allowed pattern '$pattern'"
            return 0
        fi
    done

    log_warn "Fix suggestion does not match any allowed patterns (proceeding with caution)"
    return 0
}

# 테스트 실행
run_tests() {
    log_info "Running cargo test..."
    if ! cargo test --manifest-path "$SERVER_DIR/Cargo.toml" 2>&1; then
        log_error "cargo test failed"
        return 1
    fi

    log_info "Running cargo clippy..."
    if ! cargo clippy --manifest-path "$SERVER_DIR/Cargo.toml" -- -D warnings 2>&1; then
        log_error "cargo clippy failed"
        return 1
    fi

    log_info "All tests passed"
    return 0
}

# Claude Code CLI로 수정 적용
apply_fix() {
    local fix_suggestion="$1"
    local target="${2:-codes/server/src}"
    local error_code="${3:-UNKNOWN}"

    log_info "Applying fix using Claude Code CLI..."

    # 수정 프롬프트 구성
    local prompt="다음 수정을 적용해주세요:

**수정 내용**: $fix_suggestion
**대상 위치**: $target
**에러 코드**: $error_code

**수정 규칙**:
1. 설정 값 조정 (타임아웃, 재시도 횟수 등)만 허용
2. 로깅 개선만 허용
3. 간단한 버그 수정 (오타, null 체크)만 허용
4. 의존성 패치 업데이트만 허용

**금지 사항**:
- 아키텍처 변경 금지
- 비즈니스 로직 수정 금지
- 보안 관련 코드 수정 금지

변경 사항을 최소화하고, 기존 코드 스타일을 유지하세요.
수정 완료 후 간단한 요약을 출력하세요."

    # Claude Code CLI 호출
    # --dangerously-skip-permissions: CI/CD 자동화 환경에서 사용자 입력 없이 실행하기 위해 필요
    # 보안 주의: 이 플래그는 자동화된 파이프라인 내에서만 사용되며, 수정 범위는 validate_fix_scope()에서 제한됨
    # 주의: -p/--print 플래그는 출력만 하므로 사용하지 않음 (실제 파일 수정 필요)
    local claude_output
    local claude_exit_code=0
    claude_output=$(claude --dangerously-skip-permissions "$prompt" 2>&1) || claude_exit_code=$?

    if [ "$claude_exit_code" -ne 0 ]; then
        log_error "Claude Code CLI failed with exit code $claude_exit_code"
        log_error "Claude output: $claude_output"
        return 1
    fi

    log_info "Claude Code CLI completed successfully"
    log_info "Claude output: ${claude_output:0:500}..."  # 처음 500자만 로깅

    return 0
}

# 메인 실행
main() {
    # 인자 검증
    if [ $# -lt 1 ]; then
        usage
    fi

    local diagnostic_json="$1"

    # JSON 파싱
    if ! echo "$diagnostic_json" | jq -e '.' >/dev/null 2>&1; then
        log_error "Invalid JSON input"
        usage
    fi

    # 필수 필드 추출
    local auto_fixable
    auto_fixable=$(echo "$diagnostic_json" | jq -r '.auto_fixable // false')

    if [ "$auto_fixable" != "true" ]; then
        log_info "auto_fixable is not true, skipping fix"
        exit 0
    fi

    # error_code를 먼저 추출 (알림에 사용)
    local error_code
    error_code=$(echo "$diagnostic_json" | jq -r '.error_code // "UNKNOWN"')

    local fix_suggestion
    fix_suggestion=$(echo "$diagnostic_json" | jq -r '.fix_suggestion // empty')

    if [ -z "$fix_suggestion" ]; then
        log_error "fix_suggestion is required when auto_fixable is true"
        send_notification "warning" \
            "Auto-Fix Skipped: Missing fix_suggestion" \
            "auto_fixable=true이지만 fix_suggestion이 없습니다.\n**에러 코드**: $error_code\n\n진단 결과를 확인해주세요." \
            "$error_code"
        exit 1
    fi

    local severity
    severity=$(echo "$diagnostic_json" | jq -r '.severity // "info"')

    local root_cause
    root_cause=$(echo "$diagnostic_json" | jq -r '.root_cause // "Unknown"')

    local target
    target=$(echo "$diagnostic_json" | jq -r '.target // "codes/server/src"')

    # 크로스 플랫폼 배타적 락 획득
    if ! acquire_lock "$LOCK_FILE" "$LOCK_TIMEOUT"; then
        log_error "Could not acquire lock (another auto-fix instance running?)"
        exit 1
    fi

    # 일일 제한 확인
    if ! check_daily_limit; then
        send_notification "warning" \
            "Auto-Fix Skipped: Daily Limit" \
            "일일 Auto-Fix PR 제한 ($MAX_DAILY_PRS개)에 도달했습니다.\n내일 다시 시도됩니다." \
            "$error_code"
        exit 1
    fi

    # 수정 범위 검증
    if ! validate_fix_scope "$fix_suggestion"; then
        send_notification "warning" \
            "Auto-Fix Skipped: Scope Violation" \
            "수정 내용이 허용 범위를 벗어납니다.\n**수정 제안**: $fix_suggestion" \
            "$error_code"
        exit 1
    fi

    # 작업 트리 clean 상태 확인 (미커밋 변경이 새 브랜치에 섞이는 것 방지)
    if ! git -C "$PROJECT_ROOT" diff --quiet || ! git -C "$PROJECT_ROOT" diff --cached --quiet; then
        log_error "Working tree is not clean. Please commit or stash changes before running auto-fix."
        send_notification "warning" \
            "Auto-Fix Skipped: Dirty Worktree" \
            "작업 트리에 커밋되지 않은 변경이 있습니다.\n변경 사항을 커밋하거나 stash한 후 다시 시도하세요." \
            "$error_code"
        exit 1
    fi

    # 현재 브랜치 저장
    local original_branch
    original_branch=$(git -C "$PROJECT_ROOT" rev-parse --abbrev-ref HEAD)

    # 브랜치명 생성 (error_code sanitize하여 특수문자 처리)
    local timestamp
    timestamp=$(date +%Y%m%d%H%M%S)
    local safe_error_code
    safe_error_code=$(sanitize_branch_name "$error_code")
    local branch_name="fix/auto-${safe_error_code}-${timestamp}"

    log_info "Creating fix branch: $branch_name"

    # 기본 브랜치에서 새 브랜치 생성
    git -C "$PROJECT_ROOT" fetch origin "$BASE_BRANCH" 2>/dev/null || true
    git -C "$PROJECT_ROOT" checkout -b "$branch_name" "origin/$BASE_BRANCH"

    # 수정 적용
    if ! apply_fix "$fix_suggestion" "$target" "$error_code"; then
        cleanup_branch "$branch_name" "$original_branch"
        send_notification "critical" \
            "Auto-Fix Failed: Apply Error" \
            "Claude Code CLI 수정 적용 실패\n**에러 코드**: $error_code\n**수정 제안**: $fix_suggestion" \
            "$error_code"
        exit 1
    fi

    # 변경 사항 확인
    if ! git -C "$PROJECT_ROOT" diff --quiet; then
        log_info "Changes detected, staging files..."
        # 안전을 위해 codes/server/ 경로만 스테이징 (의도치 않은 파일 포함 방지)
        git -C "$PROJECT_ROOT" add "codes/server/"
    else
        log_warn "No changes detected after applying fix"
        cleanup_branch "$branch_name" "$original_branch"
        send_notification "info" \
            "Auto-Fix Skipped: No Changes" \
            "수정 적용 후 변경 사항이 없습니다.\n**에러 코드**: $error_code" \
            "$error_code"
        exit 0
    fi

    # 테스트 실행
    if ! run_tests; then
        cleanup_branch "$branch_name" "$original_branch"
        send_notification "critical" \
            "Auto-Fix Failed: Tests Failed" \
            "테스트 실패로 Auto-Fix가 중단되었습니다.\n**에러 코드**: $error_code\n**수정 제안**: $fix_suggestion" \
            "$error_code"
        exit 1
    fi

    # 커밋 생성
    local commit_message="fix: [$error_code] $root_cause

Auto-Fix applied by AI diagnostic system.

## Changes
$fix_suggestion

## Severity
$severity

Co-Authored-By: Claude <noreply@anthropic.com>"

    git -C "$PROJECT_ROOT" commit -m "$commit_message"

    # 원격에 푸시 (실패 시 정리 로직 실행 보장)
    log_info "Pushing branch to origin..."
    local push_output
    local push_exit_code=0
    push_output=$(git -C "$PROJECT_ROOT" push -u origin "$branch_name" 2>&1) || push_exit_code=$?

    if [ "$push_exit_code" -ne 0 ]; then
        log_error "Git push failed with exit code $push_exit_code: $push_output"
        cleanup_branch "$branch_name" "$original_branch"
        send_notification "critical" \
            "Auto-Fix Failed: Push Error" \
            "브랜치 푸시 실패\n**브랜치**: $branch_name\n**에러**: $push_output" \
            "$error_code"
        exit 1
    fi

    log_info "Branch pushed successfully"

    # Draft PR 생성
    log_info "Creating Draft PR..."
    local pr_body="## Summary
- **Error Code**: \`$error_code\`
- **Severity**: $severity
- **Root Cause**: $root_cause

## Changes Applied
$fix_suggestion

## Test Results
- cargo test: Passed
- cargo clippy: Passed

## Review Checklist
- [ ] 변경 사항이 수정 허용 범위 내인지 확인
- [ ] 비즈니스 로직 변경이 없는지 확인
- [ ] 보안 관련 수정이 없는지 확인
- [ ] 테스트가 충분한지 확인

---
> This PR was automatically generated by the AI Auto-Fix system.
> Please review carefully before merging.

Generated with [Claude Code](https://claude.com/claude-code)"

    local pr_url
    local pr_exit_code
    pr_url=$(gh pr create \
        --base "$BASE_BRANCH" \
        --head "$branch_name" \
        --title "fix: [$error_code] $root_cause" \
        --body "$pr_body" \
        --draft \
        --label "auto-fix" \
        --label "ai-generated" 2>&1) || pr_exit_code=$?

    if [ "${pr_exit_code:-0}" -ne 0 ]; then
        log_error "Failed to create PR: $pr_url"
        # PR 생성 실패해도 브랜치는 유지 (수동 PR 생성 가능)
        send_notification "warning" \
            "Auto-Fix: PR Creation Failed" \
            "브랜치는 생성되었지만 PR 생성에 실패했습니다.\n**브랜치**: $branch_name\n**에러**: $pr_url" \
            "$error_code"
        git -C "$PROJECT_ROOT" checkout "$original_branch"
        exit 1
    fi

    log_info "Draft PR created successfully: $pr_url"

    # 일일 카운트 증가
    increment_daily_count

    # 원래 브랜치로 복귀
    git -C "$PROJECT_ROOT" checkout "$original_branch"

    # 성공 알림
    send_notification "info" \
        "Auto-Fix PR Created" \
        "**Error Code**: $error_code\n**Root Cause**: $root_cause\n**PR**: $pr_url\n\n리뷰 후 머지해주세요." \
        "$error_code"

    echo "$pr_url"
}

main "$@"
