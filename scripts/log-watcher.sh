#!/bin/bash
# scripts/log-watcher.sh - 로그 감시 및 에러 감지
# set -e 제거: jq 파싱 실패 시에도 계속 진행

# 스크립트 위치 기반 절대 경로 설정
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# ============== 설정 파일 체크 ==============
# 자동화가 비활성화되어 있으면 종료
check_automation_enabled() {
    local config_file="$PROJECT_ROOT/.claude/config/automation.config.yaml"

    if [ ! -f "$config_file" ]; then
        echo "[$(date)] WARN: Config file not found, automation disabled by default"
        return 1
    fi

    # Python config-loader 사용
    if ! python3 "$SCRIPT_DIR/config-loader.py" --check log_watcher 2>/dev/null; then
        echo "[$(date)] INFO: Automation is disabled in config"
        return 1
    fi

    return 0
}

# 자동화 활성화 체크
if ! check_automation_enabled; then
    echo "[$(date)] Exiting: automation disabled"
    exit 0
fi

# 설정 (절대 경로 사용)
LOG_DIR="${LOG_DIR:-$PROJECT_ROOT/codes/server/logs}"
STATE_DIR="${STATE_DIR:-$PROJECT_ROOT/logs/.state}"
DEDUP_WINDOW=300  # 5분
LOCK_TIMEOUT=10   # 락 대기 시간 (초)

# ============== 에러 심각도 분류 ==============
# Critical: 즉시 알림 + AI 진단 + GitHub Issue
# - 서버 내부 오류, AI 서비스 장애
CRITICAL_CODES="COMMON500|AI5001|AI5002|AI5003|AI5031"

# Warning: 알림만 (AI 진단 없음)
# - 인증 오류 (4xx 클라이언트 에러)
WARNING_CODES="AUTH4001|AUTH4002|AUTH4003|AUTH4004|AUTH4005"

# Info: 로그만 (알림 없음)
# - 400대 클라이언트 에러, 비즈니스 로직 에러
# (CRITICAL, WARNING에 해당하지 않는 모든 에러)

# 심각도 판단 함수
get_error_severity() {
    case "$1" in
        COMMON500|AI5001|AI5002|AI5003|AI5031) echo "critical" ;;
        AUTH4001|AUTH4002|AUTH4003|AUTH4004|AUTH4005) echo "warning" ;;
        *) echo "info" ;;
    esac
}

# 상태 디렉토리 생성
mkdir -p "$STATE_DIR"

# 오늘 로그 파일
TODAY=$(date +%Y-%m-%d)
LOG_FILE="$LOG_DIR/server.log.${TODAY}"

# 날짜별 상태 파일 (로그 로테이션 대응)
STATE_FILE="$STATE_DIR/log-watcher-state-${TODAY}"
DEDUP_FILE="$STATE_DIR/log-watcher-dedup-${TODAY}"
LOCK_FILE="$STATE_DIR/log-watcher.lock"

# 오래된 상태 파일 정리 (7일 이상)
find "$STATE_DIR" -name "log-watcher-*" -mtime +7 -delete 2>/dev/null || true

# 크로스 플랫폼 sha256 함수 (macOS/Linux 호환)
sha256_hash() {
    if command -v sha256sum &>/dev/null; then
        sha256sum | cut -d' ' -f1
    elif command -v shasum &>/dev/null; then
        shasum -a 256 | cut -d' ' -f1
    elif command -v openssl &>/dev/null; then
        openssl dgst -sha256 | awk '{print $NF}'
    else
        # fallback: md5 사용 (중복 방지 목적에는 충분)
        md5 2>/dev/null || md5sum | cut -d' ' -f1
    fi
}

# 크로스 플랫폼 락 획득 함수 (macOS/Linux 호환)
acquire_lock() {
    local lock_file="$1"
    local timeout="${2:-10}"

    if command -v flock &>/dev/null; then
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
        trap 'rmdir "$lock_dir" 2>/dev/null' EXIT
    fi
}

# 배타적 락 획득
if ! acquire_lock "$LOCK_FILE" "$LOCK_TIMEOUT"; then
    echo "[$(date)] ERROR: Could not acquire lock (another instance running?)" >&2
    exit 1
fi

# 상태 파일 초기화
touch "$STATE_FILE" "$DEDUP_FILE"

if [ ! -f "$LOG_FILE" ]; then
    echo "[$(date)] Log file not found: $LOG_FILE"
    exit 0
fi

# 현재 로그 파일의 inode 확인 (파일 교체 감지용)
CURRENT_INODE=$(stat -f%i "$LOG_FILE" 2>/dev/null || stat -c%i "$LOG_FILE" 2>/dev/null)
SAVED_INODE=$(cat "$STATE_FILE.inode" 2>/dev/null || echo "")

# inode가 변경되었으면 새 파일로 간주하고 처음부터 읽기
if [ -n "$SAVED_INODE" ] && [ "$CURRENT_INODE" != "$SAVED_INODE" ]; then
    echo "[$(date)] Log file rotated (inode changed), resetting state"
    echo "0" > "$STATE_FILE"
fi
echo "$CURRENT_INODE" > "$STATE_FILE.inode"

# 마지막 처리 라인
LAST_LINE=$(cat "$STATE_FILE" 2>/dev/null || echo 0)
[ -z "$LAST_LINE" ] && LAST_LINE=0
CURRENT_LINES=$(wc -l < "$LOG_FILE")
CURRENT_LINES=${CURRENT_LINES:-0}

# 파일이 truncate된 경우 (같은 inode지만 라인 수 감소) 처음부터 읽기
if [ "$CURRENT_LINES" -lt "$LAST_LINE" ]; then
    echo "[$(date)] Log file truncated (lines: $LAST_LINE -> $CURRENT_LINES), resetting state"
    LAST_LINE=0
fi

if [ "$CURRENT_LINES" -le "$LAST_LINE" ]; then
    echo "[$(date)] No new lines to process"
    exit 0
fi

echo "[$(date)] Processing lines $((LAST_LINE + 1)) to $CURRENT_LINES"

# 오래된 중복 기록 정리
NOW=$(date +%s)
if [ -f "$DEDUP_FILE" ]; then
    # Tab 구분자 사용, 3번째 필드(타임스탬프)가 윈도우 내인 것만 유지
    while IFS=$'\t' read -r fingerprint timestamp; do
        if [ -n "$timestamp" ] && [ $((NOW - timestamp)) -lt $DEDUP_WINDOW ]; then
            echo -e "${fingerprint}\t${timestamp}"
        fi
    done < "$DEDUP_FILE" > "${DEDUP_FILE}.tmp" 2>/dev/null || true
    mv "${DEDUP_FILE}.tmp" "$DEDUP_FILE" 2>/dev/null || true
fi

# 에러 카운터
ERROR_COUNT=0
ALERT_COUNT=0

# 새 라인 처리 (프로세스 치환으로 본 셸에서 실행하여 변수 유지)
while read -r line; do
    # 빈 라인 스킵
    [ -z "$line" ] && continue

    # JSON 유효성 검사 (jq 실패해도 계속 진행)
    if ! echo "$line" | jq -e '.' >/dev/null 2>&1; then
        # JSON이 아닌 라인은 스킵 (스택 트레이스 등)
        continue
    fi

    # JSON 파싱 (각 필드별로 개별 처리하여 부분 실패 허용)
    LEVEL=$(echo "$line" | jq -r '.level // empty' 2>/dev/null || echo "")

    if [ "$LEVEL" = "ERROR" ]; then
        ERROR_COUNT=$((ERROR_COUNT + 1))

        ERROR_CODE=$(echo "$line" | jq -r '.fields.error_code // "UNKNOWN"' 2>/dev/null || echo "UNKNOWN")
        MESSAGE=$(echo "$line" | jq -r '.message // "No message"' 2>/dev/null || echo "No message")
        TARGET=$(echo "$line" | jq -r '.target // "unknown"' 2>/dev/null || echo "unknown")
        REQUEST_ID=$(echo "$line" | jq -r '.fields.request_id // "N/A"' 2>/dev/null || echo "N/A")

        # 에러 심각도 판단
        ERROR_SEVERITY=$(get_error_severity "$ERROR_CODE")
        echo "[$(date)] Error detected: $ERROR_CODE (severity: $ERROR_SEVERITY)"

        # Info 레벨은 로그만 남기고 알림 없음
        if [ "$ERROR_SEVERITY" = "info" ]; then
            echo "[$(date)] Info-level error, logging only: $ERROR_CODE"
            continue
        fi

        # Fingerprint 생성 (SHA256 해시로 delimiter 문제 회피, macOS 호환)
        FINGERPRINT=$(echo -n "${ERROR_CODE}|${TARGET}" | sha256_hash)

        # 중복 체크 (Tab 구분자 사용)
        LAST_SEEN=""
        if [ -f "$DEDUP_FILE" ]; then
            LAST_SEEN=$(grep "^${FINGERPRINT}"$'\t' "$DEDUP_FILE" 2>/dev/null | cut -f2)
        fi

        if [ -n "$LAST_SEEN" ] && [ $((NOW - LAST_SEEN)) -lt $DEDUP_WINDOW ]; then
            echo "[$(date)] Skipping duplicate: $ERROR_CODE ($TARGET)"
            continue
        fi

        # 중복 기록 갱신 (atomic write)
        {
            grep -v "^${FINGERPRINT}"$'\t' "$DEDUP_FILE" 2>/dev/null || true
            echo -e "${FINGERPRINT}\t${NOW}"
        } > "${DEDUP_FILE}.tmp"
        mv "${DEDUP_FILE}.tmp" "$DEDUP_FILE"

        # Warning 레벨: 간단한 알림만 (AI 진단 없음)
        if [ "$ERROR_SEVERITY" = "warning" ]; then
            echo "[$(date)] Warning-level error, sending simple alert: $ERROR_CODE"
            WARN_MSG="**위치**: $TARGET
**Request ID**: $REQUEST_ID

$MESSAGE"
            if "$SCRIPT_DIR/discord-alert.sh" "warning" \
                "⚠️ [$ERROR_CODE] Warning" \
                "$WARN_MSG" \
                "$ERROR_CODE"; then
                ALERT_COUNT=$((ALERT_COUNT + 1))
            fi
            continue
        fi

        # Critical 레벨: AI 진단 + 상세 알림 + GitHub Issue
        echo "[$(date)] Critical error, running full diagnostic: $ERROR_CODE"

        # 비용 제한 체크 (시간 창 기반: 1시간 내 10회 제한)
        RATE_LIMIT_FILE="/tmp/diagnostic-rate-limit"
        HOUR_AGO=$((NOW - 3600))

        # 오래된 항목 제거 및 최근 호출 수 계산
        RECENT_CALLS=0
        if [ -f "$RATE_LIMIT_FILE" ]; then
            # 1시간 내 호출만 유지하여 임시 파일에 저장
            while read -r timestamp; do
                if [ -n "$timestamp" ] && [ "$timestamp" -gt "$HOUR_AGO" ] 2>/dev/null; then
                    echo "$timestamp"
                    RECENT_CALLS=$((RECENT_CALLS + 1))
                fi
            done < "$RATE_LIMIT_FILE" > "${RATE_LIMIT_FILE}.tmp" 2>/dev/null || true
            mv "${RATE_LIMIT_FILE}.tmp" "$RATE_LIMIT_FILE" 2>/dev/null || true
        fi

        if [ "$RECENT_CALLS" -ge 10 ]; then
            echo "[$(date)] Rate limit exceeded ($RECENT_CALLS/10 calls in last hour), sending basic critical alert"
            if "$SCRIPT_DIR/discord-alert.sh" "critical" \
                "🚨 [$ERROR_CODE] Critical Error (진단 제한 초과)" \
                "**Location**: $TARGET\n**Request ID**: $REQUEST_ID\n\n$MESSAGE" \
                "$ERROR_CODE"; then
                ALERT_COUNT=$((ALERT_COUNT + 1))
            fi
            continue
        fi

        # Diagnostic Agent 호출
        DIAGNOSTIC=$(python3 "$SCRIPT_DIR/diagnostic-agent.py" "$line" 2>/dev/null)

        # 진단 결과 JSON 유효성 검증
        if [ -z "$DIAGNOSTIC" ] || ! echo "$DIAGNOSTIC" | jq -e '.' > /dev/null 2>&1; then
            echo "[$(date)] Diagnostic returned invalid JSON, sending basic alert"
            DIAG_FAIL_MSG="**Location**: $TARGET
**Request ID**: $REQUEST_ID

$MESSAGE"
            if "$SCRIPT_DIR/discord-alert.sh" "critical" \
                "🚨 [$ERROR_CODE] Critical Error (진단 실패)" \
                "$DIAG_FAIL_MSG" \
                "$ERROR_CODE"; then
                ALERT_COUNT=$((ALERT_COUNT + 1))
            fi
            continue
        fi

        if echo "$DIAGNOSTIC" | jq -e '.error' > /dev/null 2>&1; then
            # 진단 실패 - 기본 알림
            echo "[$(date)] Diagnostic failed, sending basic alert"
            BASIC_MSG="**Location**: $TARGET
**Request ID**: $REQUEST_ID

$MESSAGE"
            if "$SCRIPT_DIR/discord-alert.sh" "critical" \
                "🚨 [$ERROR_CODE] Critical Error" \
                "$BASIC_MSG" \
                "$ERROR_CODE"; then
                ALERT_COUNT=$((ALERT_COUNT + 1))
            fi
        else
            # 진단 성공 - 상세 알림
            ROOT_CAUSE=$(echo "$DIAGNOSTIC" | jq -r '.root_cause // "분석 중"')
            RECOMMENDATIONS=$(echo "$DIAGNOSTIC" | jq -r '.recommendations[0].action // "검토 필요"')
            AUTO_FIXABLE=$(echo "$DIAGNOSTIC" | jq -r '.auto_fixable // false')

            echo "[$(date)] Diagnostic success, sending detailed alert"
            DIAG_MSG="**근본 원인**: $ROOT_CAUSE

**권장 조치**: $RECOMMENDATIONS

**위치**: $TARGET"
            if "$SCRIPT_DIR/discord-alert.sh" "critical" \
                "🔍 [$ERROR_CODE] AI 진단 완료" \
                "$DIAG_MSG" \
                "$ERROR_CODE"; then
                ALERT_COUNT=$((ALERT_COUNT + 1))
            fi

            # GitHub Issue 자동 생성
            DIAGNOSTIC_WITH_CODE=$(echo "$DIAGNOSTIC" | jq --arg ec "$ERROR_CODE" '. + {error_code: $ec}')
            echo "[$(date)] Creating GitHub Issue for: $ERROR_CODE"
            "$SCRIPT_DIR/create-issue.sh" "$DIAGNOSTIC_WITH_CODE" || true

            # Auto-Fix 시도 (auto_fixable인 경우만)
            if [ "$AUTO_FIXABLE" = "true" ]; then
                echo "[$(date)] Attempting Auto-Fix for: $ERROR_CODE"
                "$SCRIPT_DIR/auto-fix.sh" "$DIAGNOSTIC_WITH_CODE" || true
            fi
        fi
    fi
done < <(tail -n +$((LAST_LINE + 1)) "$LOG_FILE")

# 현재 라인 수 저장
echo "$CURRENT_LINES" > "$STATE_FILE"
echo "[$(date)] State saved: $CURRENT_LINES lines (errors: $ERROR_COUNT, alerts: $ALERT_COUNT)"
