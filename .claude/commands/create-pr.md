# /create-pr - 이슈 기반 PR 생성 명령어

GitHub 이슈를 읽고 CI를 통과하는 형식으로 PR을 생성합니다.

## 사용법

```
/create-pr [이슈번호]
```

### 파라미터

| 파라미터 | 설명 | 기본값 |
|----------|------|--------|
| 이슈번호 | PR과 연결할 GitHub 이슈 번호 | 필수 |

### 예시

```bash
# 이슈 #108 기반으로 PR 생성
/create-pr 108
```

## 실행 단계

### 1단계: 이슈 정보 수집

```bash
gh issue view {이슈번호} --json number,title,body,labels --jq '{number, title, body, labels: [.labels[].name]}'
```

### 2단계: 이슈 라벨로 PR 타이틀 type 결정

| 이슈 라벨 | PR type |
|-----------|---------|
| `feat` | `feat` |
| `bug`, `fix` | `fix` |
| `refactor` | `refactor` |
| `test` | `test` |
| `ci/cd` | `chore` |
| `documentation` | `docs` |
| 기타/없음 | `chore` |

### 3단계: PR 타이틀 생성

**형식**: `type: 설명`

**주의사항 (CI 실패 방지)**:
- 깃모지(이모지)를 절대 붙이지 않는다
- `action-semantic-pull-request`가 타이틀을 파싱하므로 `type:` 앞에 아무것도 붙이면 안 된다
- 이슈 제목에 이모지가 있으면 제거하고 PR 타이틀에 사용한다

```
# 좋은 예
feat: 회고방 초대 코드 조회 API 추가

# 나쁜 예 (CI 실패)
✨ feat: 회고방 초대 코드 조회 API 추가
🐛 fix: 버그 수정
```

### 4단계: 변경사항 분석

```bash
# 현재 브랜치의 변경 파일 확인
git diff dev --name-status

# 커밋 로그 확인
git log dev..HEAD --oneline
```

### 5단계: PR 생성

```bash
gh pr create \
  --title "type: 설명" \
  --base dev \
  --body "$(cat <<'EOF'
# ☝️Issue Number
- #{이슈번호}

## 📌 Summary / 개요
- {이슈 내용 기반 요약}

## 🔁 Changes / 변경 사항
### Added
- `{파일 경로}`: {설명}

### Modified
- `{파일 경로}`: {설명}

### Deleted
- `{파일 경로}`: {설명}

## ✅ Test Plan
- [ ] 단위 테스트 통과
- [ ] 통합 테스트 통과

## ☑️ Checklist
- [ ] `cargo fmt --check` 통과
- [ ] `cargo clippy -- -D warnings` 통과
- [ ] `cargo test` 통과
- [ ] API 문서 업데이트 (해당시)
- [ ] 리뷰 문서 작성 (`docs/reviews/`)

## 🔗 Related Issues
- Closes #{이슈번호}
EOF
)"
```

### 6단계: 결과 출력

PR 생성 후 URL을 출력합니다.

## 주의사항

- PR 타이틀에 **깃모지/이모지 금지** (`action-semantic-pull-request` CI 실패 원인)
- PR 타이틀은 반드시 `type: 설명` 형식
- 허용 type: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`
- base 브랜치는 `dev`
- 이슈의 TODO 항목을 PR Summary에 반영
- `Closes #이슈번호`로 이슈 자동 닫기 연결
