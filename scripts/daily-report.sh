#!/bin/bash
# scripts/daily-report.sh - 일일 서버 성능 리포트
# 매일 9시에 Cron으로 실행되어 전날 로그를 분석하고 Discord로 리포트 전송

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="${LOG_DIR:-$PROJECT_DIR/codes/server/logs}"

# 어제 날짜 (리포트 대상)
if [[ "$(uname)" == "Darwin" ]]; then
    YESTERDAY=$(date -v-1d +%Y-%m-%d)
else
    YESTERDAY=$(date -d "yesterday" +%Y-%m-%d)
fi

LOG_FILE="$LOG_DIR/server.log.$YESTERDAY"

# 로그 파일 존재 확인
if [ ! -f "$LOG_FILE" ]; then
    echo "[$(date)] INFO: No log file for $YESTERDAY ($LOG_FILE)"
    exit 0
fi

# jq 설치 확인
if ! command -v jq &> /dev/null; then
    echo "[$(date)] ERROR: jq is required but not installed" >&2
    exit 1
fi

# duration_ms 추출 (request completed 로그에서)
DURATIONS=$(jq -r 'select(.fields.message == "request completed") | .fields.duration_ms // empty' "$LOG_FILE" 2>/dev/null | sort -n)

if [ -z "$DURATIONS" ]; then
    echo "[$(date)] INFO: No request logs found for $YESTERDAY"
    exit 0
fi

# 총 요청 수
TOTAL_REQUESTS=$(echo "$DURATIONS" | wc -l | tr -d ' ')

# 퍼센타일 계산
calc_percentile() {
    local data="$1"
    local percentile="$2"
    local count
    count=$(echo "$data" | wc -l | tr -d ' ')
    local index=$(echo "$count $percentile" | awk '{printf "%d", ($1 * $2 / 100) + 0.5}')
    if [ "$index" -lt 1 ]; then index=1; fi
    if [ "$index" -gt "$count" ]; then index=$count; fi
    echo "$data" | sed -n "${index}p"
}

P50=$(calc_percentile "$DURATIONS" 50)
P95=$(calc_percentile "$DURATIONS" 95)
P99=$(calc_percentile "$DURATIONS" 99)

# 에러 수 (status >= 500)
ERROR_COUNT=$(jq -r 'select(.fields.message == "request completed") | select(.fields.status >= 500) | .fields.status' "$LOG_FILE" 2>/dev/null | wc -l | tr -d ' ')

# 에러율 계산
if [ "$TOTAL_REQUESTS" -gt 0 ]; then
    ERROR_RATE=$(echo "$ERROR_COUNT $TOTAL_REQUESTS" | awk '{printf "%.1f", ($1 / $2) * 100}')
else
    ERROR_RATE="0.0"
fi

# CRITICAL 에러 수 (AI5xxx, COMMON500)
CRITICAL_COUNT=$(jq -r 'select(.level == "ERROR") | select(.fields.error_code != null) | .fields.error_code' "$LOG_FILE" 2>/dev/null | grep -cE "(COMMON500|AI5[0-9]{3})" || true)

# CPU 사용률 (현재 스냅샷)
if [[ "$(uname)" == "Darwin" ]]; then
    CPU_USAGE=$(top -l 1 | grep "CPU usage" | awk '{print $3}' | tr -d '%')
else
    CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | awk '{printf "%.1f", $2 + $4}')
fi

# 메모리 사용률
if [[ "$(uname)" == "Darwin" ]]; then
    MEM_USAGE=$(memory_pressure 2>/dev/null | grep "System-wide memory free percentage" | awk '{printf "%.1f", 100 - $NF}' || echo "N/A")
else
    MEM_USAGE=$(free | awk '/Mem:/ {printf "%.1f", ($3/$2) * 100}')
fi

# 리포트 메시지 구성
REPORT_TITLE="📊 일일 서버 리포트 ($YESTERDAY)"
REPORT_MESSAGE="**요청**
총 ${TOTAL_REQUESTS}건

**응답시간**
P50  ${P50}ms
P95  ${P95}ms
P99  ${P99}ms

**에러**
에러율  ${ERROR_RATE}% (${ERROR_COUNT}건)
CRITICAL  ${CRITICAL_COUNT}건

**현재 시스템 상태**
CPU  ${CPU_USAGE}%
Memory  ${MEM_USAGE}%"

# Discord 전송
"$SCRIPT_DIR/discord-alert.sh" "info" "$REPORT_TITLE" "$REPORT_MESSAGE" "DAILY_REPORT"

echo "[$(date)] Daily report sent for $YESTERDAY (requests=$TOTAL_REQUESTS, p95=${P95}ms, errors=$ERROR_COUNT)"
