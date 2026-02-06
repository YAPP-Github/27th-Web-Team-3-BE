#!/bin/bash
# scripts/setup-cron.sh - Cron job 설정
# 모니터링 스크립트들의 스케줄을 등록합니다.

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
LOG_DIR="$PROJECT_DIR/logs"

# 로그 디렉토리 생성
mkdir -p "$LOG_DIR"

# 기존 모니터링 관련 cron 제거
crontab -l 2>/dev/null | grep -v "log-watcher.sh" | grep -v "daily-report.sh" | grep -v "perf-alert.sh" > /tmp/crontab.tmp || true

# 환경변수 로드 + PATH 설정 공통 prefix
ENV_PREFIX="cd $PROJECT_DIR && export PATH=/usr/local/bin:/usr/bin:\$PATH && [ -f .env ] && set -a && . ./.env && set +a"

# 1. 에러 AI 진단 (매일 9시)
echo "0 9 * * * $ENV_PREFIX && ./scripts/log-watcher.sh >> $LOG_DIR/ai-monitor.log 2>&1" >> /tmp/crontab.tmp

# 2. 일일 성능 리포트 (매일 9시 5분 - log-watcher 이후 실행)
echo "5 9 * * * $ENV_PREFIX && ./scripts/daily-report.sh >> $LOG_DIR/daily-report.log 2>&1" >> /tmp/crontab.tmp

# 3. 즉시 성능 알림 (매분)
echo "* * * * * $ENV_PREFIX && ./scripts/perf-alert.sh >> $LOG_DIR/perf-alert.log 2>&1" >> /tmp/crontab.tmp

# crontab 적용
crontab /tmp/crontab.tmp
rm /tmp/crontab.tmp

echo "Cron jobs installed:"
echo ""
crontab -l | grep -E "(log-watcher|daily-report|perf-alert)"
echo ""
echo "Logs:"
echo "  AI 진단:     $LOG_DIR/ai-monitor.log"
echo "  일일 리포트: $LOG_DIR/daily-report.log"
echo "  즉시 알림:   $LOG_DIR/perf-alert.log"
