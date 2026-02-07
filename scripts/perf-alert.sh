#!/bin/bash
# scripts/perf-alert.sh - 즉시 성능 알림
# 매분 Cron으로 실행되어 임계값 초과 시 Discord 알림 전송
# 경량 스크립트: AI 호출 없음, 시스템 지표만 체크

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="${LOG_DIR:-$PROJECT_DIR/codes/server/logs}"
STATE_DIR="/tmp/perf-alert"

mkdir -p "$STATE_DIR"

# 임계값 설정
CPU_THRESHOLD=80
MEM_THRESHOLD=85
ERROR_BURST_THRESHOLD=10
CONSECUTIVE_REQUIRED=3
DEDUP_MINUTES=10

# 상태 파일
CPU_COUNT_FILE="$STATE_DIR/cpu_count"
MEM_COUNT_FILE="$STATE_DIR/mem_count"
DEDUP_FILE="$STATE_DIR/dedup"

# 중복 알림 체크 (같은 지표 알림은 DEDUP_MINUTES 간격으로만)
check_dedup() {
    local metric="$1"
    local now
    now=$(date +%s)

    if [ -f "$DEDUP_FILE" ]; then
        local last
        last=$(grep "^${metric}:" "$DEDUP_FILE" 2>/dev/null | cut -d: -f2 || echo "0")
        if [ -n "$last" ] && [ $((now - last)) -lt $((DEDUP_MINUTES * 60)) ]; then
            return 1  # 중복, 알림 보내지 않음
        fi
    fi

    # dedup 기록 업데이트
    if [ -f "$DEDUP_FILE" ]; then
        grep -v "^${metric}:" "$DEDUP_FILE" > "$DEDUP_FILE.tmp" 2>/dev/null || true
        mv "$DEDUP_FILE.tmp" "$DEDUP_FILE"
    fi
    echo "${metric}:${now}" >> "$DEDUP_FILE"
    return 0  # 알림 전송 가능
}

# 연속 카운트 관리
increment_count() {
    local file="$1"
    local count=0
    if [ -f "$file" ]; then
        count=$(cat "$file")
    fi
    count=$((count + 1))
    echo "$count" > "$file"
    echo "$count"
}

reset_count() {
    local file="$1"
    echo "0" > "$file"
}

# CPU 사용률 체크
if [[ "$(uname)" == "Darwin" ]]; then
    CPU_USAGE=$(top -l 1 -n 0 | grep "CPU usage" | awk '{gsub(/%/, "", $3); print $3}')
else
    CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | awk '{printf "%.0f", $2 + $4}')
fi

CPU_INT=${CPU_USAGE%.*}  # 소수점 제거하여 정수 비교
if [ "${CPU_INT:-0}" -ge "$CPU_THRESHOLD" ]; then
    CPU_COUNT=$(increment_count "$CPU_COUNT_FILE")
    if [ "$CPU_COUNT" -ge "$CONSECUTIVE_REQUIRED" ]; then
        if check_dedup "cpu"; then
            "$SCRIPT_DIR/discord-alert.sh" "warning" \
                "⚠️ CPU 사용률 경고" \
                "CPU ${CPU_USAGE}% (임계값 ${CPU_THRESHOLD}%, ${CPU_COUNT}분 연속 초과)" \
                "PERF_CPU"
        fi
    fi
else
    reset_count "$CPU_COUNT_FILE"
fi

# 메모리 사용률 체크
if [[ "$(uname)" == "Darwin" ]]; then
    MEM_USAGE=$(memory_pressure 2>/dev/null | grep "System-wide memory free percentage" | awk '{printf "%.0f", 100 - $NF}' || echo "0")
else
    MEM_USAGE=$(free | awk '/Mem:/ {printf "%.0f", ($3/$2) * 100}')
fi

MEM_INT=${MEM_USAGE%.*}  # 소수점 제거하여 정수 비교
if [ "${MEM_INT:-0}" -ge "$MEM_THRESHOLD" ]; then
    MEM_COUNT=$(increment_count "$MEM_COUNT_FILE")
    if [ "$MEM_COUNT" -ge "$CONSECUTIVE_REQUIRED" ]; then
        if check_dedup "mem"; then
            "$SCRIPT_DIR/discord-alert.sh" "warning" \
                "⚠️ 메모리 사용률 경고" \
                "Memory ${MEM_USAGE}% (임계값 ${MEM_THRESHOLD}%, ${MEM_COUNT}분 연속 초과)" \
                "PERF_MEM"
        fi
    fi
else
    reset_count "$MEM_COUNT_FILE"
fi

# 최근 1분 ERROR 급증 체크
TODAY=$(date +%Y-%m-%d)
LOG_FILE="$LOG_DIR/server.log.$TODAY"

if [ -f "$LOG_FILE" ]; then
    if [[ "$(uname)" == "Darwin" ]]; then
        ONE_MIN_AGO=$(date -v-1M -u +%Y-%m-%dT%H:%M)
    else
        ONE_MIN_AGO=$(date -u -d "1 minute ago" +%Y-%m-%dT%H:%M)
    fi

    RECENT_ERRORS=$(grep "\"level\":\"ERROR\"" "$LOG_FILE" 2>/dev/null | \
        grep "\"timestamp\":\"${ONE_MIN_AGO}" 2>/dev/null | \
        wc -l | tr -d ' ' || true)

    if [ "$RECENT_ERRORS" -ge "$ERROR_BURST_THRESHOLD" ]; then
        if check_dedup "error_burst"; then
            "$SCRIPT_DIR/discord-alert.sh" "critical" \
                "🚨 에러 급증 감지" \
                "최근 1분간 ERROR ${RECENT_ERRORS}건 발생 (임계값 ${ERROR_BURST_THRESHOLD}건)" \
                "PERF_ERROR_BURST"
        fi
    fi
fi
