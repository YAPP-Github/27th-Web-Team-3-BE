#!/bin/bash
# scripts/run-tests.sh
#
# 테스트 실행 및 결과 파싱 스크립트
# 테스트 결과를 분석하여 요약을 출력합니다.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT/codes/server"

# 테스트 실행 및 결과 저장
cargo test 2>&1 | tee /tmp/test-results.log

# 결과 파싱
PASSED=$(grep -c "ok$" /tmp/test-results.log || echo "0")
FAILED=$(grep -c "FAILED$" /tmp/test-results.log || echo "0")
IGNORED=$(grep -c "ignored$" /tmp/test-results.log || echo "0")

echo "================================================"
echo "테스트 결과 요약"
echo "================================================"
echo "통과: $PASSED"
echo "실패: $FAILED"
echo "무시: $IGNORED"
echo "================================================"

# 실패 시 상세 정보 출력
if [ "$FAILED" -gt 0 ]; then
  echo ""
  echo "실패한 테스트 목록:"
  grep "FAILED" /tmp/test-results.log
  exit 1
fi
