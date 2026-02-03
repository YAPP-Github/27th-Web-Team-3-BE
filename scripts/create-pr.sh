#!/bin/bash
# scripts/create-pr.sh - GitHub PR 자동 생성
# AI 자동화 파이프라인에서 생성된 코드 변경사항을 자동으로 PR로 생성합니다.

set -euo pipefail

# 스크립트 위치 기반 절대 경로 설정
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# 기본 설정
DEFAULT_BASE_BRANCH="dev"
BASE_BRANCH="${BASE_BRANCH:-$DEFAULT_BASE_BRANCH}"

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
Usage: $0 [OPTIONS]

GitHub PR을 자동으로 생성합니다.

Options:
  -t, --type TYPE         PR 타입 (FEAT|FIX|REFACTOR|DOCS|TEST) (필수)
  -T, --title TITLE       PR 제목 (필수, AI- prefix 없이)
  -b, --body BODY         PR 본문 (미지정 시 자동 생성)
  -B, --body-file FILE    PR 본문 파일 경로
  -l, --labels LABELS     추가 라벨 (쉼표 구분)
  -r, --reviewers LIST    리뷰어 목록 (쉼표 구분)
  --base BRANCH           베이스 브랜치 (기본값: dev)
  --draft                 Draft PR로 생성
  --dry-run               실제 PR 생성 없이 테스트 실행
  -h, --help              도움말 출력

Environment:
  BASE_BRANCH             베이스 브랜치 (기본값: dev)
  GITHUB_REPO             GitHub 저장소 (예: owner/repo)
  DISCORD_WEBHOOK_URL     Discord 알림 웹훅 URL (선택)

Examples:
  # 기능 추가 PR 생성
  $0 -t FEAT -T "회고 좋아요 토글 API 구현"

  # 버그 수정 Draft PR 생성
  $0 -t FIX -T "회고 삭제 시 500 에러 수정" --draft

  # 커스텀 본문과 리뷰어 지정
  $0 -t REFACTOR -T "AI 서비스 모듈 분리" -B pr-body.md -r "team-lead,reviewer1"

  # 라벨 추가
  $0 -t TEST -T "통합 테스트 추가" -l "urgent,backend"

PR 타입:
  FEAT      새 기능 추가
  FIX       버그 수정
  REFACTOR  리팩토링
  DOCS      문서 변경
  TEST      테스트 추가
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

# PR 타입 유효성 검사
validate_type() {
    local type="$1"
    case "$type" in
        FEAT|FIX|REFACTOR|DOCS|TEST)
            return 0
            ;;
        *)
            log_error "유효하지 않은 PR 타입: $type"
            log_error "허용된 타입: FEAT, FIX, REFACTOR, DOCS, TEST"
            exit 1
            ;;
    esac
}

# 변경된 파일 기반 라벨 자동 할당
get_auto_labels() {
    local base_branch="$1"
    local labels=("ai-generated")  # AI 생성 PR 기본 라벨

    # 변경된 파일 목록 가져오기
    local changed_files
    changed_files=$(git diff --name-only "origin/${base_branch}...HEAD" 2>/dev/null || git diff --name-only "${base_branch}...HEAD" 2>/dev/null || echo "")

    if [ -z "$changed_files" ]; then
        log_warn "변경된 파일을 감지할 수 없습니다."
        echo "${labels[*]}"
        return
    fi

    # 경로 기반 라벨 할당
    if echo "$changed_files" | grep -q "^codes/server/src/domain/"; then
        labels+=("domain")
    fi

    if echo "$changed_files" | grep -q "^codes/server/tests/"; then
        labels+=("test")
    fi

    if echo "$changed_files" | grep -q "^docs/"; then
        labels+=("documentation")
    fi

    if echo "$changed_files" | grep -q "^\.github/"; then
        labels+=("ci/cd")
    fi

    echo "${labels[*]}"
}

# 변경사항 요약 생성
generate_changes_summary() {
    local base_branch="$1"

    # 변경된 파일 상태 추출
    local added_files modified_files deleted_files
    added_files=$(git diff --name-status "origin/${base_branch}...HEAD" 2>/dev/null | grep '^A' | awk '{print $2}' || echo "")
    modified_files=$(git diff --name-status "origin/${base_branch}...HEAD" 2>/dev/null | grep '^M' | awk '{print $2}' || echo "")
    deleted_files=$(git diff --name-status "origin/${base_branch}...HEAD" 2>/dev/null | grep '^D' | awk '{print $2}' || echo "")

    # 커밋 메시지 추출
    local commit_messages
    commit_messages=$(git log "origin/${base_branch}..HEAD" --pretty=format:"- %s" 2>/dev/null || echo "")

    # Added 섹션
    local added_section=""
    if [ -n "$added_files" ]; then
        added_section="### Added\n"
        while IFS= read -r file; do
            [ -n "$file" ] && added_section+="- \`$file\`\n"
        done <<< "$added_files"
    else
        added_section="### Added\n- (없음)\n"
    fi

    # Modified 섹션
    local modified_section=""
    if [ -n "$modified_files" ]; then
        modified_section="### Modified\n"
        while IFS= read -r file; do
            [ -n "$file" ] && modified_section+="- \`$file\`\n"
        done <<< "$modified_files"
    else
        modified_section="### Modified\n- (없음)\n"
    fi

    # Deleted 섹션
    local deleted_section=""
    if [ -n "$deleted_files" ]; then
        deleted_section="### Deleted\n"
        while IFS= read -r file; do
            [ -n "$file" ] && deleted_section+="- \`$file\`\n"
        done <<< "$deleted_files"
    else
        deleted_section="### Deleted\n- (없음)\n"
    fi

    # Summary 섹션
    local summary_section=""
    if [ -n "$commit_messages" ]; then
        summary_section="## Summary\n$commit_messages\n\n"
    else
        summary_section="## Summary\n- 변경사항 요약 없음\n\n"
    fi

    echo -e "${summary_section}## Changes\n${added_section}\n${modified_section}\n${deleted_section}"
}

# PR 본문 템플릿 생성
generate_pr_body() {
    local base_branch="$1"
    local pr_type="$2"

    local changes_summary
    changes_summary=$(generate_changes_summary "$base_branch")

    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)

    cat << EOF
$changes_summary
## Test Plan
- [ ] 단위 테스트 통과
- [ ] 통합 테스트 통과
- [ ] 수동 테스트 (필요시)

## Checklist
- [ ] \`cargo fmt --check\` 통과
- [ ] \`cargo clippy -- -D warnings\` 통과
- [ ] \`cargo test\` 통과
- [ ] API 문서 업데이트 (해당시)
- [ ] 리뷰 문서 작성 (\`docs/reviews/\`)

## Related Issues
<!-- 관련 이슈가 있다면 아래에 추가해주세요 -->
<!-- - Closes #이슈번호 -->

## AI Generation Info
- Generated by: Claude Code
- Phase: 4 (PR Creation)
- Timestamp: $timestamp

---
Generated with [Claude Code](https://claude.com/claude-code)
EOF
}

# 라벨 존재 확인 및 생성
ensure_labels_exist() {
    local repo="$1"
    shift
    local labels=("$@")

    for label in "${labels[@]}"; do
        # 라벨 존재 확인
        if ! gh label list --repo "$repo" --search "$label" --json name --jq '.[].name' 2>/dev/null | grep -q "^${label}$"; then
            log_info "라벨 생성 중: $label"

            # 라벨 색상 설정
            local color
            case "$label" in
                "ai-generated")
                    color="7057FF"  # 보라색
                    ;;
                "domain")
                    color="0E8A16"  # 초록색
                    ;;
                "test")
                    color="FBCA04"  # 노란색
                    ;;
                "documentation")
                    color="0075CA"  # 파란색
                    ;;
                "ci/cd")
                    color="D93F0B"  # 주황색
                    ;;
                "hotfix")
                    color="B60205"  # 빨간색
                    ;;
                *)
                    color="C5DEF5"  # 연한 파란색
                    ;;
            esac

            gh label create "$label" --repo "$repo" --color "$color" --force 2>/dev/null || true
        fi
    done
}

# Discord 알림 전송
send_notification() {
    local severity="$1"
    local title="$2"
    local message="$3"
    local error_code="${4:-PR-CREATE}"

    if [ -x "$SCRIPT_DIR/discord-alert.sh" ] && [ -n "${DISCORD_WEBHOOK_URL:-}" ]; then
        "$SCRIPT_DIR/discord-alert.sh" "$severity" "$title" "$message" "$error_code" || true
    fi
}

# 메인 실행
main() {
    local pr_type=""
    local pr_title=""
    local pr_body=""
    local body_file=""
    local extra_labels=""
    local reviewers=""
    local is_draft=false
    local dry_run=false

    # 인자 파싱
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -t|--type)
                pr_type="$2"
                shift 2
                ;;
            -T|--title)
                pr_title="$2"
                shift 2
                ;;
            -b|--body)
                pr_body="$2"
                shift 2
                ;;
            -B|--body-file)
                body_file="$2"
                shift 2
                ;;
            -l|--labels)
                extra_labels="$2"
                shift 2
                ;;
            -r|--reviewers)
                reviewers="$2"
                shift 2
                ;;
            --base)
                BASE_BRANCH="$2"
                shift 2
                ;;
            --draft)
                is_draft=true
                shift
                ;;
            --dry-run)
                dry_run=true
                shift
                ;;
            -h|--help)
                usage
                ;;
            *)
                log_error "알 수 없는 옵션: $1"
                usage
                ;;
        esac
    done

    # 필수 인자 검증
    if [ -z "$pr_type" ]; then
        log_error "PR 타입(-t)은 필수입니다."
        usage
    fi

    if [ -z "$pr_title" ]; then
        log_error "PR 제목(-T)은 필수입니다."
        usage
    fi

    # 타입 유효성 검사
    validate_type "$pr_type"

    # gh CLI 확인
    check_gh_cli

    # 저장소 결정
    local repo
    if [ -n "${GITHUB_REPO:-}" ]; then
        repo="$GITHUB_REPO"
    else
        repo=$(gh repo view --json nameWithOwner --jq '.nameWithOwner' 2>/dev/null || echo "")
        if [ -z "$repo" ]; then
            log_error "GitHub 저장소를 결정할 수 없습니다. GITHUB_REPO 환경변수를 설정하거나 git 저장소 내에서 실행해주세요."
            exit 1
        fi
    fi

    log_info "Target repository: $repo"
    log_info "Base branch: $BASE_BRANCH"

    # 현재 브랜치 확인
    local current_branch
    current_branch=$(git -C "$PROJECT_ROOT" rev-parse --abbrev-ref HEAD)

    if [ "$current_branch" = "$BASE_BRANCH" ]; then
        log_error "현재 브랜치($current_branch)가 베이스 브랜치($BASE_BRANCH)와 동일합니다."
        log_error "PR을 생성하려면 다른 브랜치에서 실행해주세요."
        exit 1
    fi

    log_info "Current branch: $current_branch"

    # origin/BASE_BRANCH 존재 여부 확인 및 fetch
    if ! git rev-parse --verify "origin/${BASE_BRANCH}" &>/dev/null; then
        log_warn "origin/${BASE_BRANCH}가 로컬에 없습니다. fetch 시도 중..."
        if ! git fetch origin "$BASE_BRANCH" 2>/dev/null; then
            log_warn "origin에서 fetch 실패. 로컬 브랜치로 fallback..."
            if ! git rev-parse --verify "$BASE_BRANCH" &>/dev/null; then
                log_error "베이스 브랜치($BASE_BRANCH)를 찾을 수 없습니다."
                exit 1
            fi
        fi
    fi

    # 변경사항 확인 (origin 우선, 로컬 fallback)
    local diff_count
    if git rev-parse --verify "origin/${BASE_BRANCH}" &>/dev/null; then
        diff_count=$(git diff "origin/${BASE_BRANCH}...HEAD" --name-only 2>/dev/null | wc -l | tr -d ' ')
    else
        diff_count=$(git diff "${BASE_BRANCH}...HEAD" --name-only 2>/dev/null | wc -l | tr -d ' ')
    fi

    if [ "$diff_count" -eq 0 ]; then
        log_error "베이스 브랜치($BASE_BRANCH)와 비교하여 변경사항이 없습니다."
        exit 1
    fi

    log_info "Changed files: $diff_count"

    # PR 제목 생성 (semantic 형식: type: description)
    # pr-check.yml의 semantic-pull-request action과 호환되도록 lowercase type 사용
    local semantic_type
    case "$pr_type" in
        FEAT) semantic_type="feat" ;;
        FIX) semantic_type="fix" ;;
        REFACTOR) semantic_type="refactor" ;;
        DOCS) semantic_type="docs" ;;
        TEST) semantic_type="test" ;;
    esac
    local full_title="${semantic_type}: [AI] ${pr_title}"

    # PR 본문 결정
    local final_body
    if [ -n "$body_file" ] && [ -f "$body_file" ]; then
        final_body=$(cat "$body_file")
        log_info "PR 본문을 파일에서 로드: $body_file"
    elif [ -n "$pr_body" ]; then
        final_body="$pr_body"
        log_info "PR 본문을 인자에서 사용"
    else
        final_body=$(generate_pr_body "$BASE_BRANCH" "$pr_type")
        log_info "PR 본문 자동 생성"
    fi

    # 자동 라벨 생성
    local auto_labels
    auto_labels=$(get_auto_labels "$BASE_BRANCH")
    log_info "Auto-detected labels: $auto_labels"

    # 최종 라벨 목록 생성
    local all_labels="$auto_labels"
    if [ -n "$extra_labels" ]; then
        all_labels="$auto_labels $extra_labels"
    fi

    # 라벨을 배열로 변환
    local labels_array=()
    for label in $all_labels; do
        # 쉼표로 구분된 라벨 처리
        IFS=',' read -ra split_labels <<< "$label"
        for l in "${split_labels[@]}"; do
            [ -n "$l" ] && labels_array+=("$l")
        done
    done

    # 중복 제거
    local unique_labels
    unique_labels=$(printf '%s\n' "${labels_array[@]}" | sort -u | tr '\n' ' ')

    log_info "Final labels: $unique_labels"

    # 라벨 생성 확인
    ensure_labels_exist "$repo" ${unique_labels}

    # 라벨 문자열 생성 (쉼표 구분)
    local labels_str
    labels_str=$(echo "$unique_labels" | tr ' ' ',' | sed 's/,$//')

    # Dry-run 모드
    if [ "$dry_run" = true ]; then
        log_info "=== DRY RUN MODE ==="
        log_info "Title: $full_title"
        log_info "Base: $BASE_BRANCH"
        log_info "Head: $current_branch"
        log_info "Labels: $labels_str"
        log_info "Reviewers: ${reviewers:-none}"
        log_info "Draft: $is_draft"
        log_info "=== PR Body ==="
        echo "$final_body"
        log_info "=== END DRY RUN ==="
        exit 0
    fi

    # 원격 브랜치 확인 및 푸시
    if ! git ls-remote --exit-code --heads origin "$current_branch" &>/dev/null; then
        log_info "원격 브랜치가 없습니다. 푸시 중..."
        git -C "$PROJECT_ROOT" push -u origin "$current_branch"
    fi

    log_info "Creating PR..."

    # PR 생성 실행
    local pr_url
    local pr_exit_code=0

    # PR 생성 명령 인자 배열 구성 (공백/특수문자 안전 처리)
    local -a pr_args=(
        --repo "$repo"
        --base "$BASE_BRANCH"
        --head "$current_branch"
        --title "$full_title"
        --body "$final_body"
        --label "$labels_str"
    )

    # 리뷰어 추가 (값이 있을 때만)
    if [ -n "$reviewers" ]; then
        pr_args+=(--reviewer "$reviewers")
    fi

    # Draft 옵션 (명시적 true 체크)
    if [ "$is_draft" = true ]; then
        pr_args+=(--draft)
    fi

    pr_url=$(gh pr create "${pr_args[@]}" 2>&1) || pr_exit_code=$?

    if [ "$pr_exit_code" -ne 0 ]; then
        log_error "PR 생성 실패: $pr_url"
        send_notification "critical" \
            "PR Creation Failed" \
            "PR 생성에 실패했습니다.\n**Branch**: $current_branch\n**Error**: $pr_url" \
            "PR-CREATE"
        exit 1
    fi

    log_info "PR created successfully: $pr_url"

    # 성공 알림
    send_notification "info" \
        "New AI-Generated PR" \
        "**Title**: $full_title\n**Branch**: $current_branch -> $BASE_BRANCH\n**PR**: $pr_url" \
        "PR-CREATE"

    # PR URL 출력
    echo "$pr_url"
}

# 스크립트 실행
main "$@"
