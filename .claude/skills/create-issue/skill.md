# GitHub Issue 생성 Skill

GitHub 이슈를 프로젝트 템플릿에 맞춰 생성하는 스킬입니다.

## 언제 이 스킬을 사용하나요?

- 새로운 기능/버그/리팩토링 등의 이슈를 생성할 때
- `gh issue create` 명령을 사용할 때

## 이슈 타입별 양식

### Feature (기능 개발)

```bash
gh issue create \
  --title "기능 설명" \
  --label "feat" \
  --body "$(cat <<'EOF'
# Title

- 자세한 개요 작성

# TODO

- [ ] 투두 내용 작성

# etc

- 참고자료 등 기타 내용 작성
EOF
)"
```

### Bug Fix (버그 수정)

```bash
gh issue create \
  --title "버그 설명" \
  --label "bug,fix" \
  --body "$(cat <<'EOF'
# Title

- 자세한 개요 작성

# TODO

- [ ] 투두 내용 작성

# etc

- 참고자료 등 기타 내용 작성
EOF
)"
```

### Refactor (리팩토링)

```bash
gh issue create \
  --title "리팩토링 설명" \
  --label "refactor" \
  --body "$(cat <<'EOF'
# Title

- 자세한 개요 작성

# TODO

- [ ] 투두 내용 작성

# etc

- 참고자료 등 기타 내용 작성
EOF
)"
```

### Test (테스트)

```bash
gh issue create \
  --title "테스트 설명" \
  --label "test" \
  --body "$(cat <<'EOF'
# Title

- 자세한 개요 작성

# TODO

- [ ] 투두 내용 작성

# etc

- 참고자료 등 기타 내용 작성
EOF
)"
```

### CI/CD (배포)

```bash
gh issue create \
  --title "CI/CD 설명" \
  --label "ci/cd" \
  --body "$(cat <<'EOF'
# Title

- 자세한 개요 작성

# TODO

- [ ] 투두 내용 작성

# etc

- 참고자료 등 기타 내용 작성
EOF
)"
```

### Docs (문서)

```bash
gh issue create \
  --title "문서 설명" \
  --label "documentation" \
  --body "$(cat <<'EOF'
# Title

- 자세한 개요 작성

# TODO

- [ ] 투두 내용 작성

# etc

- 참고자료 등 기타 내용 작성
EOF
)"
```

## 규칙

1. **이슈 타입에 맞는 라벨**을 반드시 지정
2. **제목은 명확하고 간결**하게 작성
3. **TODO 항목**은 구체적인 작업 단위로 분리
4. 관련 이슈가 있으면 `# etc` 섹션에 링크 추가
