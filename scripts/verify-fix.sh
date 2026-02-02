#!/bin/bash
# scripts/verify-fix.sh
#
# AI 수정 검증 스크립트
# 코드 수정 후 모든 검증 단계를 실행합니다.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT/codes/server"

echo "=== AI 수정 검증 시작 ==="
echo "대상 브랜치: $(git branch --show-current)"
echo ""

# 1단계: 문법 검증
echo "[1/5] 문법 검증..."
if ! cargo check 2>/dev/null; then
  echo "FAILED: 문법 오류 발견"
  exit 1
fi
echo "PASSED"

# 2단계: 컴파일 검증
echo "[2/5] 컴파일 검증..."
if ! cargo build 2>/dev/null; then
  echo "FAILED: 컴파일 실패"
  exit 1
fi
echo "PASSED"

# 3단계: 테스트 검증
echo "[3/5] 테스트 검증..."
set +e
TEST_OUTPUT=$(cargo test 2>&1)
TEST_EXIT_CODE=$?
set -e
if [ $TEST_EXIT_CODE -ne 0 ] || echo "$TEST_OUTPUT" | grep -q "FAILED"; then
  echo "FAILED: 테스트 실패"
  echo "$TEST_OUTPUT" | grep -E "(FAILED|error\[)" || true
  echo ""
  echo "=== 테스트 실패 요약 ==="
  echo "종료 코드: $TEST_EXIT_CODE"
  exit 1
fi
echo "PASSED"

# 4단계: Clippy 검증
echo "[4/5] Clippy 검증..."
if ! cargo clippy -- -D warnings 2>/dev/null; then
  echo "FAILED: Clippy 경고 발견"
  exit 1
fi
echo "PASSED"

# 5단계: 포맷팅 검증
echo "[5/5] 포맷팅 검증..."
if ! cargo fmt --check 2>/dev/null; then
  echo "FAILED: 포맷팅 불일치"
  exit 1
fi
echo "PASSED"

echo ""
echo "=== 모든 검증 통과 ==="
echo "수정을 적용해도 안전합니다."
