#!/bin/bash
# scripts/quality-check.sh
#
# 통합 품질 검사 스크립트
# 포맷팅, 린트, 빌드, 테스트를 순차적으로 실행합니다.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT/codes/server"

echo "=== 품질 검사 시작 ==="

# 1. 포맷팅 검사
echo "[1/4] 포맷팅 검사..."
if ! cargo fmt --check; then
  echo "포맷팅 필요 - 자동 적용 중..."
  cargo fmt
  echo "포맷팅 완료"
fi

# 2. Clippy 검사
echo "[2/4] Clippy 검사..."
cargo clippy -- -D warnings

# 3. 빌드 검사
echo "[3/4] 빌드 검사..."
cargo build

# 4. 테스트 실행
echo "[4/4] 테스트 실행..."
cargo test

echo "=== 품질 검사 완료: 모든 검사 통과 ==="
