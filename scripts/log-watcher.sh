#!/bin/bash
# scripts/log-watcher.sh - 로그 감시 및 에러 감지
# set -e 제거: jq 파싱 실패 시에도 계속 진행

# 설정
LOG_DIR="${LOG_DIR:-./logs}"
STATE_DIR="${STATE_DIR:-./logs/.state}"
DEDUP_WINDOW=300  # 5분
LOCK_TIMEOUT=10   # 락 대기 시간 (초)

# 상태 디렉토리 생성
mkdir -p "$STATE_DIR"

# 오늘 로그 파일
TODAY=$(date +%Y-%m-%d)
LOG_FILE="$LOG_DIR/server.${TODAY}.log"

# 날짜별 상태 파일 (로그 로테이션 대응)
STATE_FILE="$STATE_DIR/log-watcher-state-${TODAY}"
DEDUP_FILE="$STATE_DIR/log-watcher-dedup-${TODAY}"
LOCK_FILE="$STATE_DIR/log-watcher.lock"

# 오래된 상태 파일 정리 (7일 이상)
find "$STATE_DIR" -name "log-watcher-*" -mtime +7 -delete 2>/dev/null || true

# flock을 사용한 배타적 락 획득
exec 200>"$LOCK_FILE"
if ! flock -w "$LOCK_TIMEOUT" 200; then
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
CURRENT_LINES=$(wc -l < "$LOG_FILE")

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

# 새 라인 처리
tail -n +$((LAST_LINE + 1)) "$LOG_FILE" | while read -r line; do
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

        # Fingerprint 생성 (SHA256 해시로 delimiter 문제 회피)
        FINGERPRINT=$(echo -n "${ERROR_CODE}|${TARGET}" | sha256sum | cut -d' ' -f1)

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

        # 비용 제한 체크 (진단 호출 전 필수)
        if ! python3 -c "
import time
from pathlib import Path

RATE_LIMIT_FILE = Path('/tmp/diagnostic-rate-limit')
MAX_CALLS_PER_HOUR = 10

now = time.time()
hour_ago = now - 3600

if not RATE_LIMIT_FILE.exists():
    RATE_LIMIT_FILE.write_text(str(now))
    exit(0)  # 허용

calls = [float(t) for t in RATE_LIMIT_FILE.read_text().split('\n') if t]
recent_calls = [t for t in calls if t > hour_ago]

if len(recent_calls) >= MAX_CALLS_PER_HOUR:
    exit(1)  # 제한 초과

recent_calls.append(now)
RATE_LIMIT_FILE.write_text('\n'.join(str(t) for t in recent_calls))
exit(0)  # 허용
"; then
            # 비용 제한 초과 - 기본 알림만 발송
            echo "[$(date)] Rate limit exceeded, skipping diagnostic"
            if ./scripts/discord-alert.sh "critical" \
                "🚨 [$ERROR_CODE] Error Detected (진단 제한 초과)" \
                "**Location**: $TARGET\n**Request ID**: $REQUEST_ID\n\n$MESSAGE" \
                "$ERROR_CODE"; then
                ALERT_COUNT=$((ALERT_COUNT + 1))
            fi
            continue
        fi

        # Diagnostic Agent 호출 (비용 제한 통과 후)
        echo "[$(date)] Running diagnostic for: $ERROR_CODE"
        DIAGNOSTIC=$(python3 ./scripts/diagnostic-agent.py "$line" 2>/dev/null)

        if echo "$DIAGNOSTIC" | jq -e '.error' > /dev/null 2>&1; then
            # 진단 실패 - 기본 알림
            echo "[$(date)] Diagnostic failed, sending basic alert"
            if ./scripts/discord-alert.sh "critical" \
                "🚨 [$ERROR_CODE] Error Detected" \
                "**Location**: $TARGET\n**Request ID**: $REQUEST_ID\n\n$MESSAGE" \
                "$ERROR_CODE"; then
                ALERT_COUNT=$((ALERT_COUNT + 1))
            fi
        else
            # 진단 성공 - 상세 알림
            SEVERITY=$(echo "$DIAGNOSTIC" | jq -r '.severity // "critical"')
            ROOT_CAUSE=$(echo "$DIAGNOSTIC" | jq -r '.root_cause // "분석 중"')
            RECOMMENDATIONS=$(echo "$DIAGNOSTIC" | jq -r '.recommendations[0].action // "검토 필요"')

            echo "[$(date)] Diagnostic success, sending detailed alert"
            if ./scripts/discord-alert.sh "$SEVERITY" \
                "🔍 [$ERROR_CODE] AI 진단 완료" \
                "**근본 원인**: $ROOT_CAUSE\n\n**권장 조치**: $RECOMMENDATIONS\n\n**위치**: $TARGET" \
                "$ERROR_CODE"; then
                ALERT_COUNT=$((ALERT_COUNT + 1))
            fi
        fi
    fi
done

# 현재 라인 수 저장
echo "$CURRENT_LINES" > "$STATE_FILE"
echo "[$(date)] State saved: $CURRENT_LINES lines (errors: $ERROR_COUNT, alerts: $ALERT_COUNT)"
