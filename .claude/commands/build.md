# /build - Rust 빌드 명령어

프로젝트를 빌드하고 결과를 보고합니다.

## 실행 단계

1. `cargo fmt` - 포맷팅 적용
2. `cargo clippy -- -D warnings` - 린트 검사
3. `cargo build` - 빌드 실행

## 실패 시

- 포맷팅 오류: 자동 수정 후 재시도
- Clippy 경고: 문제 파일과 수정 방법 제시
- 빌드 에러: 에러 메시지 분석 및 해결책 제안

## 사용 예시

```
/build          # 기본 빌드
/build release  # 릴리즈 빌드
```
