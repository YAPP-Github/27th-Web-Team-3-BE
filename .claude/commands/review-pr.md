# /review-pr - PR 코드 리뷰 명령어

PR의 변경사항을 분석하고 인라인 코드 리뷰를 작성합니다.

## 사용법

```
/review-pr [PR번호] [옵션]
```

### 파라미터

| 파라미터 | 설명 | 기본값 |
|----------|------|--------|
| PR번호 | 리뷰할 PR 번호 | 현재 브랜치의 PR |
| 옵션 | `comment`, `approve`, `request-changes` | 자동 판단 |

### 리뷰 결과 자동 판단 기준

옵션을 지정하지 않으면 리뷰 결과에 따라 자동으로 판단합니다:

| 조건 | 결과 |
|------|------|
| 🔴 Bug/Security 이슈 있음 | `request-changes` |
| 🟡 개선 필요 사항만 있음 | `comment` |
| 🟢 문제 없음 | `approve` |

### 예시

```bash
# 현재 브랜치의 PR 리뷰 (자동 판단)
/review-pr

# 특정 PR 리뷰 (자동 판단)
/review-pr 91

# 강제로 코멘트만 (승인/거절 없음)
/review-pr 91 comment

# 강제로 승인
/review-pr 91 approve

# 강제로 변경 요청
/review-pr 91 request-changes
```

## 실행 단계

### 1단계: PR 정보 수집

```bash
# PR 상세 정보
gh pr view {PR번호} --json number,title,body,author,baseRefName,headRefName,files,additions,deletions

# 변경된 파일 목록
gh pr view {PR번호} --json files --jq '.files[].path'

# PR diff 가져오기
gh pr diff {PR번호}
```

### 2단계: 변경사항 분석

각 파일에 대해:

1. **파일 타입별 분류**
   - `.rs` → Rust 코드 리뷰 규칙 적용
   - `.md` → 문서 리뷰 (오타, 링크 검증)
   - `.sh` → 쉘 스크립트 리뷰
   - `.json`/`.toml` → 설정 파일 검증

2. **Rust 코드 리뷰 체크리스트** (`.rs` 파일)
   - [ ] `unwrap()` / `expect()` 사용 여부 (테스트 제외)
   - [ ] `Result<T, AppError>` 반환 패턴
   - [ ] `#[serde(rename_all = "camelCase")]` 적용
   - [ ] 에러 처리 적절성
   - [ ] 로깅 (`tracing`) 사용
   - [ ] 테스트 존재 여부

3. **보안 체크**
   - [ ] 하드코딩된 시크릿 없음
   - [ ] SQL 인젝션 가능성
   - [ ] 입력값 검증

### 3단계: 인라인 코멘트 작성

변경된 각 라인에 대해 리뷰가 필요하면 인라인 코멘트를 작성합니다.

```bash
# 먼저 최신 커밋 SHA 가져오기
COMMIT_SHA=$(gh pr view {PR번호} --json headRefOid --jq '.headRefOid')

# 인라인 코멘트 작성 (특정 라인)
gh api repos/{owner}/{repo}/pulls/{PR번호}/comments \
  -X POST \
  -F body="코멘트 내용" \
  -F commit_id="$COMMIT_SHA" \
  -F path="파일경로" \
  -F line=줄번호 \
  -F side="RIGHT"
```

#### 코멘트 형식

```markdown
**[카테고리]** 설명

예시:
- **[Style]** `snake_case` 네이밍 규칙을 따라주세요.
- **[Bug]** 이 경우 `unwrap()`이 panic을 일으킬 수 있습니다.
- **[Suggestion]** `?` 연산자로 간결하게 작성할 수 있습니다.
- **[Security]** 하드코딩된 값을 환경변수로 이동하세요.
- **[Nit]** 불필요한 공백이 있습니다. (minor)
```

#### 카테고리

| 카테고리 | 설명 | 심각도 |
|----------|------|--------|
| `Bug` | 버그 또는 런타임 에러 가능성 | 🔴 높음 |
| `Security` | 보안 취약점 | 🔴 높음 |
| `Error Handling` | 에러 처리 누락/부적절 | 🟡 중간 |
| `Style` | 코딩 컨벤션 위반 | 🟢 낮음 |
| `Suggestion` | 개선 제안 | 🟢 낮음 |
| `Nit` | 사소한 지적 | ⚪ 매우 낮음 |
| `Question` | 질문/확인 필요 | ⚪ 정보 |

### 4단계: 전체 리뷰 제출

```bash
# 리뷰 제출 (코멘트만)
gh pr review {PR번호} --comment --body "$(cat <<'EOF'
## 코드 리뷰 요약

### ✅ 잘된 점
- ...

### 📝 개선 제안
- ...

### ⚠️ 주의 필요
- ...

---
🤖 Reviewed by Claude Code
EOF
)"

# 승인과 함께 리뷰
gh pr review {PR번호} --approve --body "LGTM! 🚀"

# 변경 요청
gh pr review {PR번호} --request-changes --body "위 코멘트 수정 후 다시 리뷰 요청해주세요."
```

## 리뷰 기준

### Rust 코드 (`codes/server/src/**/*.rs`)

| 항목 | 규칙 | 참조 |
|------|------|------|
| 에러 처리 | `unwrap()` 금지, `Result` + `?` 사용 | `.claude/rules/rust-src.md` |
| 직렬화 | `#[serde(rename_all = "camelCase")]` 필수 | `.claude/rules/api-design.md` |
| 핸들러 | 비즈니스 로직 없음, 검증만 | `.claude/rules/api-design.md` |
| 서비스 | 순수 비즈니스 로직 | `.claude/rules/rust-src.md` |

### 테스트 코드 (`codes/server/tests/**/*.rs`)

| 항목 | 규칙 |
|------|------|
| 구조 | AAA 패턴 (Arrange-Act-Assert) |
| 네이밍 | `should_*` 또는 `test_*` |
| 커버리지 | 정상 + 에러 케이스 |

### 문서 (`docs/**/*.md`)

| 항목 | 확인 사항 |
|------|----------|
| 링크 | 내부 링크 유효성 |
| 오타 | 맞춤법/문법 |
| 형식 | 마크다운 문법 |

## 출력 형식

```
========================================
PR #{번호} 코드 리뷰
========================================

📁 변경 파일: N개 (+{additions} -{deletions})

[파일1.rs]
  L42: 🔴 [Bug] unwrap() 사용 - panic 가능성
  L78: 🟡 [Suggestion] match 대신 if let 사용 권장

[파일2.md]
  L15: 🟢 [Nit] 링크 깨짐

----------------------------------------
📊 리뷰 요약
----------------------------------------
🔴 버그/보안: 1건
🟡 개선 제안: 2건
🟢 스타일/Nit: 3건

총 {N}개 인라인 코멘트 작성됨
========================================
```

## 주의사항

- 인라인 코멘트는 **변경된 라인**에만 작성 가능
- 삭제된 라인에 코멘트하려면 `side="LEFT"` 사용
- 이미 작성된 코멘트는 중복 작성하지 않음
- 리뷰 후 자동으로 PR 페이지 URL 출력
