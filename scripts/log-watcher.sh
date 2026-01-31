#!/bin/bash
# scripts/log-watcher.sh - 로그 감시 및 에러 감지

set -e

# 설정
LOG_DIR="${LOG_DIR:-./logs}"
STATE_DIR="${STATE_DIR:-./logs/.state}"  # 상태 파일을 프로젝트 내에 저장
DEDUP_WINDOW=300  # 5분

# 상태 디렉토리 생성
mkdir -p "$STATE_DIR"

# 오늘 로그 파일
TODAY=$(date +%Y-%m-%d)
LOG_FILE="$LOG_DIR/server.${TODAY}.log"

# 날짜별 상태 파일 (로그 로테이션 대응)
STATE_FILE="$STATE_DIR/log-watcher-state-${TODAY}"
DEDUP_FILE="$STATE_DIR/log-watcher-dedup-${TODAY}"

# 오래된 상태 파일 정리 (7일 이상)
find "$STATE_DIR" -name "log-watcher-*" -mtime +7 -delete 2>/dev/null || true

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

if [ "$CURRENT_LINES" -le "$LAST_LINE" ]; then
    echo "[$(date)] No new lines to process"
    exit 0
fi

echo "[$(date)] Processing lines $((LAST_LINE + 1)) to $CURRENT_LINES"

# 새 라인 처리
tail -n +$((LAST_LINE + 1)) "$LOG_FILE" | while read -r line; do
    # JSON 파싱
    LEVEL=$(echo "$line" | jq -r '.level // empty' 2>/dev/null)

    if [ "$LEVEL" = "ERROR" ]; then
        ERROR_CODE=$(echo "$line" | jq -r '.fields.error_code // "UNKNOWN"')
        MESSAGE=$(echo "$line" | jq -r '.message // "No message"')
        TARGET=$(echo "$line" | jq -r '.target // "unknown"')
        REQUEST_ID=$(echo "$line" | jq -r '.fields.request_id // "N/A"')

        # Fingerprint 생성 (중복 체크용)
        FINGERPRINT="${ERROR_CODE}:${TARGET}"

        # 중복 체크
        NOW=$(date +%s)
        LAST_SEEN=$(grep "^${FINGERPRINT}:" "$DEDUP_FILE" 2>/dev/null | cut -d: -f3)

        if [ -n "$LAST_SEEN" ] && [ $((NOW - LAST_SEEN)) -lt $DEDUP_WINDOW ]; then
            echo "[$(date)] Skipping duplicate: $FINGERPRINT"
            continue
        fi

        # 중복 기록 갱신
        grep -v "^${FINGERPRINT}:" "$DEDUP_FILE" > "${DEDUP_FILE}.tmp" 2>/dev/null || true
        echo "${FINGERPRINT}:${NOW}" >> "${DEDUP_FILE}.tmp"
        mv "${DEDUP_FILE}.tmp" "$DEDUP_FILE"

        # Discord 알림
        echo "[$(date)] Sending alert for: $ERROR_CODE"
        ./scripts/discord-alert.sh "critical" \
            "🚨 [$ERROR_CODE] Error Detected" \
            "**Location**: $TARGET\n**Request ID**: $REQUEST_ID\n\n$MESSAGE" \
            "$ERROR_CODE"
    fi
done

# 현재 라인 수 저장
echo "$CURRENT_LINES" > "$STATE_FILE"
echo "[$(date)] State saved: $CURRENT_LINES lines"
