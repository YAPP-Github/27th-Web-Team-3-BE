# CLAUDE.md - Server Project Guide

## Build & Test Commands
- Full Build/Deploy: `./build.sh` (이미지에 있는 쉘 스크립트 활용)
- Standard Build: `cargo build`
- Run Server: `cargo run`
- Test All: `cargo test`
- Lint: `cargo clippy`
- Format: `cargo fmt`

## Project Context
- **Name:** Rust Server
- **Environment:** Configured via `.env` file
- **Build System:** Custom `build.sh` script used for deployments
- **Core Logic:** Located in `src/`

## Coding Style & Rules
- **Error Handling:** Use `Result` and `anyhow` for server-side errors.
- **Environment Variables:** Always use the `.env` patterns for configuration.
- **Async:** Use `tokio` patterns for asynchronous tasks (if applicable).
- **Documentation:** Public functions should have summary doc comments.
- **Strictness:** Follow `clippy` suggestions to maintain code quality.
- **TDD:** Prioritize writing tests in `src/` before implementing complex server logic.
- **폴더구조:** 도메인형으로 해줘 (각 도메인마다 폴더안에 해주고 없다면 만들어서 해줘)

## 요약
- 구현할 때 참고한 document 같은 파일 바로 및에 아래와 같은 형식으로 정리해줘
```md
# 📊 [리포트] 구현 결과 확인

### 1. 구현 요약
* **상태:** ✅ 개발 완료 / ⚠️ 수정 중
* **설계 준수:** 설계서의 모든 예외 처리 및 규약 반영 완료.

### 2. 정상 작동 증빙 (Success Case)
* **상황:** (예: 5,000원 정상 충전 시)
* **입력 데이터:** `{"id": "user123", "amount": 5000}`
* **실제 출력값:** `{"status": "success", "currentBalance": 5000}`

### 3. 에러 대응 증빙 (Error Case)
* **시나리오 1:** (예: 잔액 부족 상황 테스트)
    - **결과:** `{"status": "fail", "reason": "INSUFFICIENT_FUNDS"}` (설계와 일치)
* **시나리오 2:** (예: 잘못된 입력값 테스트)
    - **결과:** `{"status": "fail", "reason": "INVALID_INPUT"}`

### 4. 기타 특이사항
* (예: 외부 결제 모듈 점검 시 응답이 2초 정도 지연될 수 있음)
```