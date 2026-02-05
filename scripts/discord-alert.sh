#!/bin/bash
# scripts/discord-alert.sh - Discord 알림 발송

WEBHOOK_URL="${DISCORD_WEBHOOK_URL}"
SEVERITY="$1"
TITLE="$2"
MESSAGE="$3"
ERROR_CODE="$4"

# 필수 인자 검증
if [ -z "$TITLE" ] || [ -z "$MESSAGE" ]; then
    echo "[$(date)] ERROR: Usage: $0 <severity> <title> <message> [error_code]" >&2
    exit 1
fi

# Webhook URL 검증
if [ -z "$WEBHOOK_URL" ]; then
    echo "[$(date)] ERROR: DISCORD_WEBHOOK_URL is not set" >&2
    exit 1
fi

if [[ ! "$WEBHOOK_URL" =~ ^https://discord\.com/api/webhooks/ ]] && \
   [[ ! "$WEBHOOK_URL" =~ ^https://discordapp\.com/api/webhooks/ ]]; then
    echo "[$(date)] ERROR: Invalid Discord webhook URL format" >&2
    exit 1
fi

# 색상 설정 (Discord embed color)
case "$SEVERITY" in
    critical) COLOR=15158332 ;;  # Red
    warning)  COLOR=16776960 ;;  # Yellow
    info)     COLOR=3066993 ;;   # Green
    *)        COLOR=8421504 ;;   # Gray
esac

# JSON 안전 이스케이프 함수
json_escape() {
    local str="$1"
    # 백슬래시를 먼저 이스케이프 (순서 중요)
    str="${str//\\/\\\\}"
    # 따옴표 이스케이프
    str="${str//\"/\\\"}"
    # 제어 문자 이스케이프
    str="${str//$'\n'/\\n}"
    str="${str//$'\r'/\\r}"
    str="${str//$'\t'/\\t}"
    echo "$str"
}

SAFE_TITLE=$(json_escape "$TITLE")
SAFE_MESSAGE=$(json_escape "$MESSAGE")
SAFE_ERROR_CODE=$(json_escape "$ERROR_CODE")

# HTTP 응답 코드와 본문을 함께 받기
RESPONSE=$(curl -s -w "\n%{http_code}" -H "Content-Type: application/json" \
     -X POST \
     -d "{
       \"embeds\": [{
         \"title\": \"$SAFE_TITLE\",
         \"description\": \"$SAFE_MESSAGE\",
         \"color\": $COLOR,
         \"footer\": {
           \"text\": \"Error Code: $SAFE_ERROR_CODE\"
         },
         \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
       }]
     }" \
     "$WEBHOOK_URL" 2>&1)

# 응답 파싱 (마지막 줄이 HTTP 코드)
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

# HTTP 코드 검증
if [[ ! "$HTTP_CODE" =~ ^2[0-9][0-9]$ ]]; then
    echo "[$(date)] ERROR: Discord webhook failed with HTTP $HTTP_CODE" >&2
    echo "[$(date)] Response: $BODY" >&2
    exit 1
fi

echo "[$(date)] Alert sent successfully (HTTP $HTTP_CODE)"
