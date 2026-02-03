#!/bin/bash
# scripts/rollback-fix.sh
#
# AI 수정 롤백 스크립트
# 실패한 AI 수정 브랜치를 정리합니다.
#
# 사용법: ./rollback-fix.sh <branch-name>
# 예시: ./rollback-fix.sh ai-fix/ERR-001/20260201-143025

BRANCH_NAME=$1

if [ -z "$BRANCH_NAME" ]; then
  echo "사용법: $0 <branch-name>"
  echo "예시: $0 ai-fix/ERR-001/20260201-143025"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "=== 롤백 시작: $BRANCH_NAME ==="

# 현재 브랜치 확인
CURRENT_BRANCH=$(git branch --show-current)

if [ "$CURRENT_BRANCH" = "$BRANCH_NAME" ]; then
  echo "현재 브랜치에서 롤백 중... dev로 이동합니다."
  git checkout dev
fi

# 로컬 브랜치 삭제
if git branch --list | grep -q "$BRANCH_NAME"; then
  git branch -D "$BRANCH_NAME"
  echo "로컬 브랜치 삭제 완료"
else
  echo "로컬 브랜치가 존재하지 않습니다."
fi

# 원격 브랜치 삭제 (존재하는 경우)
if git ls-remote --heads origin "$BRANCH_NAME" 2>/dev/null | grep -q "$BRANCH_NAME"; then
  git push origin --delete "$BRANCH_NAME"
  echo "원격 브랜치 삭제 완료"
else
  echo "원격 브랜치가 존재하지 않습니다."
fi

# PR 닫기 (존재하는 경우)
if command -v gh &> /dev/null; then
  PR_NUMBER=$(gh pr list --head "$BRANCH_NAME" --json number -q '.[0].number' 2>/dev/null)
  if [ -n "$PR_NUMBER" ]; then
    gh pr close "$PR_NUMBER" --comment "AI 자동 수정 롤백됨"
    echo "PR #$PR_NUMBER 닫기 완료"
  else
    echo "관련 PR이 없습니다."
  fi
else
  echo "gh CLI가 설치되어 있지 않습니다. PR 닫기를 건너뜁니다."
fi

echo "=== 롤백 완료 ==="
