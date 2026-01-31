#!/bin/bash
# scripts/discord-alert.sh - Discord 알림 발송

set -e

WEBHOOK_URL="${DISCORD_WEBHOOK_URL}"
SEVERITY="$1"
TITLE="$2"
MESSAGE="$3"
ERROR_CODE="$4"

# 색상 설정 (Discord embed color)
case "$SEVERITY" in
    critical) COLOR=15158332 ;;  # Red
    warning)  COLOR=16776960 ;;  # Yellow
    info)     COLOR=3066993 ;;   # Green
    *)        COLOR=8421504 ;;   # Gray
esac

# 메시지 이스케이프
MESSAGE=$(echo "$MESSAGE" | sed 's/"/\\"/g' | tr '\n' ' ')

curl -s -H "Content-Type: application/json" \
     -X POST \
     -d "{
       \"embeds\": [{
         \"title\": \"$TITLE\",
         \"description\": \"$MESSAGE\",
         \"color\": $COLOR,
         \"footer\": {
           \"text\": \"Error Code: $ERROR_CODE\"
         },
         \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
       }]
     }" \
     "$WEBHOOK_URL"
