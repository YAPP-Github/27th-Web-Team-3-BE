# Phase 2 (MVP): 모니터링 MVP

## 개요

| 항목 | 내용 |
|------|------|
| Phase | 2: MVP |
| 기간 | Week 3-4 |
| 목표 | Log Watcher, Discord 알림, 기본 필터링 |
| 의존성 | Phase 1 (Foundation) 완료 |

```
Phase 2 완료 상태
┌─────────────────────────────────────────────────────────────┐
│  ✅ Discord Webhook    ✅ Log Watcher    ✅ Cron 설정      │
└─────────────────────────────────────────────────────────────┘
```

## 완료 조건

- [ ] ERROR 로그 발생 시 Discord 알림 수신
- [ ] 5분 내 동일 에러 중복 알림 방지
- [ ] Cron으로 5분 간격 자동 실행

---

## 사전 조건

### 필수 도구
```bash
# macOS
brew install jq

# Ubuntu
apt-get install jq curl
```

### Discord Webhook 생성
1. Discord 서버 → 채널 설정 → 연동 → 웹후크
2. 웹후크 URL 복사
3. `.env`에 추가:
```bash
DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/xxx/yyy
```

---

## 태스크 2.1: Discord Webhook 연동

### 구현

**파일**: `scripts/discord-alert.sh`

```bash
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
```

### 테스트

```bash
chmod +x scripts/discord-alert.sh
export DISCORD_WEBHOOK_URL="https://discord.com/api/webhooks/..."

./scripts/discord-alert.sh critical "Test Alert" "This is a test message" "TEST_001"
```

### 체크리스트

- [ ] 스크립트 생성 및 실행 권한
- [ ] `.env`에 DISCORD_WEBHOOK_URL 설정
- [ ] 테스트 알림 발송 확인

---

## 태스크 2.2: Log Watcher 스크립트

### 구현

**파일**: `scripts/log-watcher.sh`

```bash
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
# - 날짜가 변경되면 새 상태 파일 사용
# - 이전 날짜 상태 파일은 무시됨
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
```

### 로그 로테이션 대응

상태 파일은 날짜별로 분리되어 저장됩니다:

```
logs/.state/
├── log-watcher-state-2025-01-31      # 라인 번호 상태
├── log-watcher-state-2025-01-31.inode # 파일 inode (교체 감지)
├── log-watcher-dedup-2025-01-31      # 중복 체크 상태
└── log-watcher-state-2025-01-30      # 이전 날짜 (7일 후 자동 삭제)
```

**동작 방식**:
1. 날짜가 변경되면 새 상태 파일 사용 (이전 상태 무시)
2. 같은 날짜에 로그 파일이 교체되면 inode 변경 감지하여 상태 리셋
3. 7일 이상 된 상태 파일은 자동 정리

### 체크리스트

- [ ] 스크립트 생성 및 실행 권한
- [ ] `LOG_DIR` 환경변수 또는 기본값 확인
- [ ] jq로 JSON 파싱 동작 확인
- [ ] 중복 제거 로직 테스트

### 수동 테스트

```bash
# 테스트 로그 생성
mkdir -p logs
echo '{"timestamp":"2025-01-31T10:00:00Z","level":"ERROR","target":"server::test","fields":{"error_code":"TEST_001"},"message":"Test error"}' >> logs/server.$(date +%Y-%m-%d).log

# Log Watcher 실행
./scripts/log-watcher.sh
```

---

## 태스크 2.3: Cron 설정

### 구현

**파일**: `scripts/setup-cron.sh`

```bash
#!/bin/bash
# scripts/setup-cron.sh - Cron job 설정

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
LOG_DIR="$PROJECT_DIR/logs"

# 로그 디렉토리 생성
mkdir -p "$LOG_DIR"

# 기존 log-watcher cron 제거
crontab -l 2>/dev/null | grep -v "log-watcher.sh" > /tmp/crontab.tmp || true

# 새 cron 추가 (5분 간격)
# - .env 파일을 source하여 환경변수 로드
# - PATH 설정으로 jq, curl 등 명령어 사용 가능하게 함
# - 로그는 프로젝트 logs/ 디렉토리에 저장 (권한 문제 방지)
echo "*/5 * * * * cd $PROJECT_DIR && export PATH=/usr/local/bin:/usr/bin:\$PATH && [ -f .env ] && export \$(grep -v '^#' .env | xargs) && ./scripts/log-watcher.sh >> $LOG_DIR/ai-monitor.log 2>&1" >> /tmp/crontab.tmp

# crontab 적용
crontab /tmp/crontab.tmp
rm /tmp/crontab.tmp

echo "Cron job installed:"
crontab -l | grep log-watcher
echo ""
echo "Log output: $LOG_DIR/ai-monitor.log"
```

### 체크리스트

- [ ] 스크립트 실행 권한
- [ ] crontab 설정 확인: `crontab -l`
- [ ] 로그 파일 경로 확인 (프로젝트 logs/ 디렉토리)
- [ ] .env 파일에 DISCORD_WEBHOOK_URL 설정 확인

---

## 트러블슈팅

### Discord 알림이 안 옴
```bash
# Webhook URL 확인
echo $DISCORD_WEBHOOK_URL

# 직접 테스트
curl -X POST -H "Content-Type: application/json" \
  -d '{"content":"test"}' "$DISCORD_WEBHOOK_URL"
```

### JSON 파싱 실패
```bash
# 로그 포맷 확인
tail -1 logs/server.*.log | jq .

# jq 설치 확인
which jq
```

### Cron이 실행 안 됨
```bash
# cron 서비스 확인
sudo service cron status

# 로그 확인 (프로젝트 logs/ 디렉토리)
tail -f logs/ai-monitor.log

# 환경변수 확인 (cron은 별도 환경이므로 .env 로드 필요)
# setup-cron.sh가 자동으로 .env를 source하도록 설정됨
```

---

## 산출물

Phase 2 완료 시:

1. **Discord 알림**
   - ERROR 발생 시 실시간 알림
   - 에러 코드, 위치, Request ID 포함

2. **중복 방지**
   - 5분 내 동일 에러 알림 1회만

3. **자동 실행**
   - Cron으로 5분 간격 모니터링

---

## 다음 Phase 연결

Phase 3에서:
- Log Watcher가 에러 감지 시 → Diagnostic Agent 호출
- 단순 알림 → AI 진단 결과 포함 알림으로 업그레이드
