#!/bin/bash
# scripts/create-issue.sh - GitHub Issue 자동 생성
# 진단 결과(JSON)를 입력으로 받아 GitHub Issue를 자동 생성합니다.
# 중복 이슈가 있으면 코멘트를 추가합니다.

set -euo pipefail

# 스크립트 위치 기반 절대 경로 설정
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# 로깅 함수
log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $*"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" >&2
}

log_warn() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARN: $*"
}

# 사용법 출력
usage() {
    cat << EOF
Usage: $0 <diagnostic_json>

진단 결과 JSON을 받아 GitHub Issue를 생성합니다.

Arguments:
  diagnostic_json  진단 결과 JSON 문자열 또는 JSON 파일 경로

Environment:
  GITHUB_REPO      GitHub 저장소 (예: owner/repo). 미설정 시 현재 저장소 사용

Example:
  $0 '{"error_code": "API_001", "severity": "critical", ...}'
  $0 /path/to/diagnostic.json

JSON 형식:
  {
    "error_code": "API_001",
    "severity": "critical|warning|info",
    "root_cause": "에러 원인 설명",
    "impact": "영향 범위",
    "recommendations": [{"effort": "low|medium|high", "action": "조치 내용"}],
    "auto_fixable": true|false
  }
EOF
    exit 1
}

# gh CLI 확인
check_gh_cli() {
    if ! command -v gh &> /dev/null; then
        log_error "gh CLI가 설치되어 있지 않습니다. https://cli.github.com/ 에서 설치해주세요."
        exit 1
    fi

    if ! gh auth status &> /dev/null; then
        log_error "gh CLI 인증이 필요합니다. 'gh auth login'을 실행해주세요."
        exit 1
    fi
}

# jq 확인
check_jq() {
    if ! command -v jq &> /dev/null; then
        log_error "jq가 설치되어 있지 않습니다."
        exit 1
    fi
}

# JSON 입력 파싱
parse_input() {
    local input="$1"

    # 파일 경로인 경우
    if [ -f "$input" ]; then
        cat "$input"
    else
        echo "$input"
    fi
}

# JSON 유효성 검사
validate_json() {
    local json="$1"

    if ! echo "$json" | jq -e '.' > /dev/null 2>&1; then
        log_error "유효하지 않은 JSON 형식입니다."
        exit 1
    fi

    # 필수 필드 확인
    local error_code
    error_code=$(echo "$json" | jq -r '.error_code // empty')
    if [ -z "$error_code" ]; then
        log_error "error_code 필드가 필요합니다."
        exit 1
    fi

    local severity
    severity=$(echo "$json" | jq -r '.severity // empty')
    if [ -z "$severity" ]; then
        log_error "severity 필드가 필요합니다."
        exit 1
    fi
}

# 심각도에 따른 우선순위 라벨 매핑
get_priority_label() {
    local severity="$1"

    case "$severity" in
        critical)
            echo "priority:critical"
            ;;
        warning)
            echo "priority:high"
            ;;
        info)
            echo "priority:low"
            ;;
        *)
            echo "priority:medium"
            ;;
    esac
}

# 심각도에 따른 이모지
get_severity_emoji() {
    local severity="$1"

    case "$severity" in
        critical)
            echo "🚨"
            ;;
        warning)
            echo "⚠️"
            ;;
        info)
            echo "ℹ️"
            ;;
        *)
            echo "🔔"
            ;;
    esac
}

# 중복 이슈 확인
find_existing_issue() {
    local error_code="$1"
    local repo="$2"

    # error_code가 제목에 포함된 열린 이슈 검색
    local issue_number
    issue_number=$(gh issue list \
        --repo "$repo" \
        --state open \
        --search "[${error_code}]" \
        --json number \
        --jq '.[0].number // empty' 2>/dev/null || echo "")

    echo "$issue_number"
}

# 라벨 확인 및 생성
ensure_labels_exist() {
    local repo="$1"
    shift
    local labels=("$@")

    for label in "${labels[@]}"; do
        if ! gh label list --repo "$repo" --search "$label" --json name --jq '.[].name' 2>/dev/null | grep -q "^${label}$"; then
            log_info "라벨 생성 중: $label"

            # 라벨 색상 설정
            local color
            case "$label" in
                "priority:critical")
                    color="B60205"  # 빨간색
                    ;;
                "priority:high")
                    color="D93F0B"  # 주황색
                    ;;
                "priority:medium")
                    color="FBCA04"  # 노란색
                    ;;
                "priority:low")
                    color="0E8A16"  # 초록색
                    ;;
                "ai-generated")
                    color="7057FF"  # 보라색
                    ;;
                "auto-fix")
                    color="1D76DB"  # 파란색
                    ;;
                "bug")
                    color="D73A4A"  # GitHub 기본 bug 색상
                    ;;
                *)
                    color="C5DEF5"  # 연한 파란색
                    ;;
            esac

            gh label create "$label" --repo "$repo" --color "$color" --force 2>/dev/null || true
        fi
    done
}

# Issue 본문 생성
generate_issue_body() {
    local json="$1"

    local error_code root_cause impact auto_fixable
    error_code=$(echo "$json" | jq -r '.error_code')
    root_cause=$(echo "$json" | jq -r '.root_cause // "분석 중"')
    impact=$(echo "$json" | jq -r '.impact // "영향 분석 중"')
    auto_fixable=$(echo "$json" | jq -r '.auto_fixable // false')

    # 권장 조치 파싱
    local recommendations_md=""
    local recommendations_count
    recommendations_count=$(echo "$json" | jq -r '.recommendations | length')

    if [ "$recommendations_count" -gt 0 ]; then
        recommendations_md="## Recommendations\n\n"
        for i in $(seq 0 $((recommendations_count - 1))); do
            local effort action
            effort=$(echo "$json" | jq -r ".recommendations[$i].effort // \"medium\"")
            action=$(echo "$json" | jq -r ".recommendations[$i].action // \"\"")

            local effort_badge
            case "$effort" in
                low)
                    effort_badge="![Effort: Low](https://img.shields.io/badge/effort-low-green)"
                    ;;
                medium)
                    effort_badge="![Effort: Medium](https://img.shields.io/badge/effort-medium-yellow)"
                    ;;
                high)
                    effort_badge="![Effort: High](https://img.shields.io/badge/effort-high-red)"
                    ;;
                *)
                    effort_badge="![Effort: Unknown](https://img.shields.io/badge/effort-unknown-gray)"
                    ;;
            esac

            recommendations_md+="- $effort_badge $action\n"
        done
    fi

    # 자동 수정 가능 여부
    local auto_fix_section=""
    if [ "$auto_fixable" = "true" ]; then
        auto_fix_section="\n## Auto-fix Available\n\nThis issue can be automatically fixed. Run the appropriate fix script or apply the suggested changes.\n"
    fi

    cat << EOF
## Root Cause

$root_cause

## Impact

$impact

$(echo -e "$recommendations_md")
$auto_fix_section
---
> This issue was automatically generated by AI diagnostic system.
> Error Code: \`$error_code\`
> Generated at: $(date -u '+%Y-%m-%d %H:%M:%S UTC')
EOF
}

# 코멘트 본문 생성
generate_comment_body() {
    local json="$1"

    local root_cause impact
    root_cause=$(echo "$json" | jq -r '.root_cause // "분석 중"')
    impact=$(echo "$json" | jq -r '.impact // "영향 분석 중"')

    # 권장 조치
    local first_recommendation
    first_recommendation=$(echo "$json" | jq -r '.recommendations[0].action // "추가 분석 필요"')

    cat << EOF
## 추가 발생 알림

동일한 에러가 다시 발생했습니다.

**발생 시간**: $(date -u '+%Y-%m-%d %H:%M:%S UTC')

### 이번 진단 결과
- **Root Cause**: $root_cause
- **Impact**: $impact
- **권장 조치**: $first_recommendation

---
> AI Diagnostic System
EOF
}

# 메인 실행
main() {
    # 인자 확인
    if [ $# -lt 1 ]; then
        usage
    fi

    local input="$1"

    # 의존성 확인
    check_gh_cli
    check_jq

    # JSON 파싱 및 검증
    local json
    json=$(parse_input "$input")
    validate_json "$json"

    # 필드 추출
    local error_code severity auto_fixable
    error_code=$(echo "$json" | jq -r '.error_code')
    severity=$(echo "$json" | jq -r '.severity')
    auto_fixable=$(echo "$json" | jq -r '.auto_fixable // false')

    log_info "Processing diagnostic for error: $error_code (severity: $severity)"

    # 저장소 결정
    local repo
    if [ -n "${GITHUB_REPO:-}" ]; then
        repo="$GITHUB_REPO"
    else
        # 현재 디렉토리의 git remote에서 추출
        repo=$(gh repo view --json nameWithOwner --jq '.nameWithOwner' 2>/dev/null || echo "")
        if [ -z "$repo" ]; then
            log_error "GitHub 저장소를 결정할 수 없습니다. GITHUB_REPO 환경변수를 설정하거나 git 저장소 내에서 실행해주세요."
            exit 1
        fi
    fi

    log_info "Target repository: $repo"

    # 라벨 준비 (bug 라벨 필수 포함, auto-fix로 통일)
    local priority_label
    priority_label=$(get_priority_label "$severity")

    local labels=("bug" "ai-generated" "$priority_label")
    if [ "$auto_fixable" = "true" ]; then
        labels+=("auto-fix")
    fi

    # 라벨 생성 확인
    ensure_labels_exist "$repo" "${labels[@]}"

    # 중복 이슈 확인
    local existing_issue
    existing_issue=$(find_existing_issue "$error_code" "$repo")

    if [ -n "$existing_issue" ]; then
        # 기존 이슈에 코멘트 추가
        log_info "Existing issue found: #$existing_issue. Adding comment..."

        local comment_body
        comment_body=$(generate_comment_body "$json")

        if gh issue comment "$existing_issue" --repo "$repo" --body "$comment_body"; then
            log_info "Comment added to issue #$existing_issue"
            echo "COMMENT_ADDED:$existing_issue"
        else
            log_error "Failed to add comment to issue #$existing_issue"
            exit 1
        fi
    else
        # 새 이슈 생성
        log_info "Creating new issue..."

        local emoji
        emoji=$(get_severity_emoji "$severity")

        local title="$emoji [$error_code] $(echo "$json" | jq -r '.root_cause // "Error detected"' | head -c 80)"
        local body
        body=$(generate_issue_body "$json")

        # 라벨을 쉼표로 연결
        local labels_str
        labels_str=$(IFS=','; echo "${labels[*]}")

        local new_issue
        new_issue=$(gh issue create \
            --repo "$repo" \
            --title "$title" \
            --body "$body" \
            --label "$labels_str" \
            2>&1)

        if [ $? -eq 0 ]; then
            log_info "Issue created: $new_issue"
            echo "ISSUE_CREATED:$new_issue"
        else
            log_error "Failed to create issue: $new_issue"
            exit 1
        fi
    fi
}

# 스크립트 실행
main "$@"
