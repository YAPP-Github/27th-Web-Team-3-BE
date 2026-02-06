#!/bin/bash
set -euo pipefail

#######################################
# GitHub 라벨 초기 설정 스크립트
# Phase 4: Issue Automation
#
# 사용법: ./scripts/setup-labels.sh
#######################################

echo "=== GitHub 라벨 설정 스크립트 ==="
echo ""

#######################################
# 사전 조건 확인
#######################################

# gh CLI 설치 확인
if ! command -v gh &> /dev/null; then
    echo "ERROR: gh CLI가 설치되지 않았습니다."
    echo "설치 방법:"
    echo "  - macOS: brew install gh"
    echo "  - Ubuntu: sudo apt install gh"
    exit 1
fi
echo "[OK] gh CLI 설치 확인"

# GitHub 인증 상태 확인
if ! gh auth status &> /dev/null; then
    echo "ERROR: GitHub 인증이 필요합니다."
    echo "실행: gh auth login"
    exit 1
fi
echo "[OK] GitHub 인증 확인"
echo ""

#######################################
# 라벨 생성 함수
#######################################
create_label() {
    local name="$1"
    local color="$2"
    local description="$3"

    echo -n "  라벨 생성: $name ... "
    if gh label create "$name" \
        --color "$color" \
        --description "$description" \
        --force 2>/dev/null; then
        echo "완료"
    else
        echo "실패 (이미 존재하거나 오류)"
    fi
}

echo "=== 라벨 생성 시작 ==="
echo ""

#######################################
# 1. AI 자동화 라벨
#######################################
echo "[1/4] AI 자동화 라벨"
create_label "ai-generated" "7057ff" "AI 모니터링 시스템이 자동 생성"
echo ""

#######################################
# 2. 우선순위 라벨
#######################################
echo "[2/4] 우선순위 라벨"
create_label "priority:critical" "b60205" "즉시 대응 필요 (P0)"
create_label "priority:high" "d93f0b" "우선 대응 필요 (P1)"
create_label "priority:medium" "fbca04" "일반 우선순위 (P2)"
create_label "priority:low" "0e8a16" "낮은 우선순위 (P3)"
echo ""

#######################################
# 3. 도메인 라벨
#######################################
echo "[3/4] 도메인 라벨"
create_label "domain:ai" "1d76db" "AI/LLM 관련 에러"
create_label "domain:auth" "5319e7" "인증/인가 관련 에러"
create_label "domain:db" "0052cc" "데이터베이스 관련 에러"
echo ""

#######################################
# 4. 자동 수정 라벨 (Phase 5 대비)
#######################################
echo "[4/4] 자동 수정 라벨"
create_label "auto-fix" "0e8a16" "AI 자동 수정 PR"
echo ""

#######################################
# 완료
#######################################
echo "=== 라벨 설정 완료 ==="
echo ""
echo "생성된 라벨:"
echo "  - ai-generated     (보라, #7057ff)"
echo "  - priority:critical (빨강, #b60205)"
echo "  - priority:high     (주황, #d93f0b)"
echo "  - priority:medium   (노랑, #fbca04)"
echo "  - priority:low      (초록, #0e8a16)"
echo "  - domain:ai         (파랑, #1d76db)"
echo "  - domain:auth       (보라, #5319e7)"
echo "  - domain:db         (진파랑, #0052cc)"
echo "  - auto-fix          (초록, #0e8a16)"
