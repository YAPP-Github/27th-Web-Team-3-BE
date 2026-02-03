#!/bin/bash
# scripts/verify-conventions.sh
#
# 코딩 컨벤션 검증 스크립트
# CLAUDE.md에 정의된 규칙을 검사합니다.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "=== 컨벤션 검증 시작 ==="

HAS_ERROR=0

# 1. unwrap/expect 사용 검사 (테스트 파일 제외)
echo "1. unwrap/expect 검사..."
UNWRAP_FOUND=$(grep -r "\.unwrap()\|\.expect(" codes/server/src/ --include="*.rs" | grep -v "test" | grep -v "#\[cfg(test)\]" || true)
if [ -n "$UNWRAP_FOUND" ]; then
  echo "WARNING: unwrap/expect 사용 감지 (테스트 외)"
  echo "$UNWRAP_FOUND"
  HAS_ERROR=1
fi

# 2. serde camelCase 검사
echo "2. serde camelCase 검사..."
# Request/Response struct 찾기
STRUCTS_WITHOUT_RENAME=$(grep -r "pub struct.*Request\|pub struct.*Response" codes/server/src/ --include="*.rs" -B 2 | grep -v "rename_all" | grep "pub struct" || true)
if [ -n "$STRUCTS_WITHOUT_RENAME" ]; then
  echo "WARNING: camelCase 누락 가능성"
  echo "$STRUCTS_WITHOUT_RENAME"
fi

# 3. Result 반환 검사
echo "3. Result 반환 타입 검사..."
# handler와 service 파일에서 pub async fn이 Result를 반환하는지 검사
FUNCS_WITHOUT_RESULT=$(grep -r "pub async fn\|pub fn" codes/server/src/domain/ --include="handler.rs" --include="service.rs" | grep -v "Result<" | grep -v "test" | grep -v "//" || true)
if [ -n "$FUNCS_WITHOUT_RESULT" ]; then
  echo "WARNING: Result 반환 타입 누락 가능성"
  echo "$FUNCS_WITHOUT_RESULT"
fi

# 4. AppError 사용 검사
echo "4. AppError 사용 검사..."
if ! grep -rq "use.*AppError" codes/server/src/domain/ --include="*.rs"; then
  echo "INFO: AppError 사용 확인 권장"
fi

echo ""
if [ "$HAS_ERROR" -eq 1 ]; then
  echo "=== 컨벤션 검증 완료 (경고 있음) ==="
  exit 1
else
  echo "=== 컨벤션 검증 완료 ==="
fi
