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
echo "*/5 * * * * cd $PROJECT_DIR && export PATH=/usr/local/bin:/usr/bin:\$PATH && [ -f .env ] && set -a && . ./.env && set +a && ./scripts/log-watcher.sh >> $LOG_DIR/ai-monitor.log 2>&1" >> /tmp/crontab.tmp

# crontab 적용
crontab /tmp/crontab.tmp
rm /tmp/crontab.tmp

echo "Cron job installed:"
crontab -l | grep log-watcher
echo ""
echo "Log output: $LOG_DIR/ai-monitor.log"
