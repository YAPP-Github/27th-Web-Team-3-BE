# /quality - 코드 품질 검사 명령어

전체 코드 품질을 점검합니다.

## 검사 항목

1. **포맷팅** - `cargo fmt --check`
2. **린트** - `cargo clippy -- -D warnings`
3. **테스트** - `cargo test`
4. **보안** - `cargo audit` (설치된 경우)

## 결과 보고

```
품질 검사 결과:
[ ] 포맷팅: PASS/FAIL
[x] 린트: PASS/FAIL (경고 N개)
[x] 테스트: PASS/FAIL (N/M 통과)
[ ] 보안: PASS/FAIL (취약점 N개)
```

## 자동 수정

- 포맷팅 문제: 자동 수정 (`cargo fmt`)
- Clippy 경고: 수정 제안 제시
- 테스트 실패: TDD 원칙에 따라 분석

## 커밋 전 필수 실행

이 명령어는 커밋 전에 실행하여 모든 품질 기준을 만족하는지 확인합니다.
