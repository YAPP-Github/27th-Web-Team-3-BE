# GitHub PR 생성 Skill

GitHub Pull Request를 CI 검증을 통과하도록 올바른 형식으로 생성하는 스킬입니다.

## 언제 이 스킬을 사용하나요?

- PR을 생성할 때
- `gh pr create` 명령을 사용할 때

## CI 검증 요구사항 (pr-check.yml)

### PR 타이틀 형식 (필수)

`action-semantic-pull-request`에 의해 검증됩니다.
반드시 아래 형식을 따라야 CI가 통과합니다:

```
type: 설명
```

**허용되는 type:**

| type | 용도 |
|------|------|
| `feat` | 새로운 기능 추가 |
| `fix` | 버그 수정 |
| `docs` | 문서 변경 |
| `refactor` | 리팩토링 |
| `test` | 테스트 추가/수정 |
| `chore` | 기타 변경 (CI, 설정 등) |

**예시:**
- `feat: 회고방 초대 코드 조회 API 추가`
- `fix: member_retro_room created_at 마이그레이션 추가`
- `chore: trigger deployment for DB schema migration`

### PR 본문 형식

`.github/PULL_REQUEST_TEMPLATE.md` 양식을 따릅니다:

```markdown
# ☝️Issue Number
- #{이슈번호}

## 📌 Summary / 개요
- {주요 변경사항 1}
- {주요 변경사항 2}

## 🔁 Changes / 변경 사항
### Added
- `{파일 경로}`: {설명}

### Modified
- `{파일 경로}`: {설명}

### Deleted
- `{파일 경로}`: {설명}

## 📸 Screenshots / 스크린샷
<!-- 필요시 첨부 -->

## ✅ Test Plan
- [ ] 단위 테스트 통과
- [ ] 통합 테스트 통과
- [ ] 수동 테스트 (필요시)

## ☑️ Checklist
- [ ] `cargo fmt --check` 통과
- [ ] `cargo clippy -- -D warnings` 통과
- [ ] `cargo test` 통과
- [ ] API 문서 업데이트 (해당시)
- [ ] 리뷰 문서 작성 (`docs/reviews/`)

## 👀 Additional Notes / 기타 사항
<!-- 추가로 이야기할 점 -->

## 🔗 Related Issues
- Closes #{이슈번호}
```

## PR 생성 명령어 예시

```bash
gh pr create \
  --title "feat: 회고방 초대 코드 조회 API 추가" \
  --base dev \
  --body "$(cat <<'EOF'
# ☝️Issue Number
- #103

## 📌 Summary / 개요
- 회고방 초대 코드 조회 API 추가

## 🔁 Changes / 변경 사항
### Added
- `codes/server/src/domain/retro_room/handler.rs`: 초대 코드 조회 핸들러 추가

### Modified
- `codes/server/src/domain/retro_room/service.rs`: 초대 코드 조회 로직 추가

## ✅ Test Plan
- [x] 단위 테스트 통과
- [x] 통합 테스트 통과

## ☑️ Checklist
- [x] `cargo fmt --check` 통과
- [x] `cargo clippy -- -D warnings` 통과
- [x] `cargo test` 통과

## 🔗 Related Issues
- Closes #103
EOF
)"
```

## 규칙

1. **PR 타이틀은 반드시 `type: 설명` 형식** (CI 실패 방지)
2. **base 브랜치는 `dev`** 가 기본
3. **관련 이슈 번호**를 Issue Number와 Related Issues에 모두 기재
4. **Checklist 항목**은 실제로 수행한 것만 체크
5. **Changes 섹션**에 실제 변경된 파일 경로를 정확히 기재
6. AI가 생성한 PR은 타이틀에 `[AI]`를 포함하면 자동으로 `ai-generated` 라벨이 붙음
