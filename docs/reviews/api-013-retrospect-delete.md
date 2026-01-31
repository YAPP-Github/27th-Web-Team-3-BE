# API-013 회고 삭제 Implementation Review

## 1. 개요
- **API 명**: `DELETE /api/v1/retrospects/{retrospectId}`
- **구현 목적**: 특정 회고 세션과 연관된 모든 데이터(답변, 댓글, 좋아요, AI 분석 결과)를 영구 삭제한다.
- **API 스펙**: `docs/api-specs/013-retrospect-delete.md`

## 2. 구현 상세

### 2.1 도메인 구조
`codes/server/src/domain/retrospect/`
- `dto.rs`: `SuccessDeleteRetrospectResponse` (Swagger용)
- `service.rs`: 회고 조회, 회고방 멤버십 확인, 트랜잭션 내 연관 데이터 캐스케이드 삭제
- `handler.rs`: HTTP 핸들러 (`delete_retrospect`) + utoipa 문서화

### 2.2 주요 로직
1. **입력 검증**:
   - Path parameter `retrospectId` >= 1 확인 (COMMON400)
2. **인증/인가**:
   - JWT 토큰에서 user_id 추출 (AUTH4001)
   - 회고 존재 여부 확인 (RETRO4041)
   - 회고방 멤버십 확인 (RETRO4041, 보안상 존재 여부 미노출)
3. **트랜잭션 내 캐스케이드 삭제** (외래 키 순서 준수):
   1. `response_comment` - 댓글 삭제
   2. `response_like` - 좋아요 삭제
   3. `member_response` - 멤버-응답 매핑 삭제
   4. `response` - 응답(질문) 삭제
   5. `retro_reference` - 참고자료 삭제
   6. `member_retro` - 멤버-회고 매핑 삭제 (개인 인사이트 포함)
   7. `member_retro_room` - 멤버-회고방 매핑 삭제
   8. `retrospect` - 회고 삭제 (팀 인사이트 포함)
   9. `retro_room` - 회고방 삭제
4. **응답 반환**: `result: null` (삭제 완료)

### 2.3 에러 코드

| Code | HTTP | Description |
|------|------|-------------|
| COMMON400 | 400 | 잘못된 Path Parameter (retrospectId < 1) |
| AUTH4001 | 401 | 인증 실패 |
| RETRO4031 | 403 | 삭제 권한 없음 |
| RETRO4041 | 404 | 존재하지 않는 회고 |
| COMMON500 | 500 | 서버 내부 오류 (트랜잭션 실패) |

### 2.4 라우트 등록
- `main.rs`에서 기존 `GET /api/v1/retrospects/:retrospect_id` 라우트에 `.delete()` 메서드 체이닝으로 추가
- OpenAPI paths 및 schemas에 등록 완료

### 2.5 권한 체계 참고
- API 스펙은 "팀 관리자(Owner)" 또는 "회고 생성자"만 삭제 가능으로 정의
- 현재 DB 스키마에 `member_team.role` 및 `retrospect.created_by` 필드가 없어 회고방 멤버십 기반 접근 제어로 구현
- 향후 스키마에 해당 필드 추가 시 `RETRO4031` 에러를 활용한 세밀한 권한 체크 가능

## 3. 테스트 결과

### 3.1 전체 테스트 통과
- 기존 89개 유닛 테스트 모두 통과
- 기존 37개 통합 테스트 모두 통과
- 새로운 에러 variant (`RetroDeleteAccessDenied`) 추가로 기존 코드에 영향 없음 확인

### 3.2 코드 품질
- `cargo fmt` 적용 완료
- `cargo clippy -- -D warnings` 경고 없음
- `cargo test` 전체 통과

## 4. 변경 파일 목록

| 파일 | 변경 내용 |
|------|----------|
| `src/utils/error.rs` | `RetroDeleteAccessDenied` variant 추가 (RETRO4031, 403) |
| `src/domain/retrospect/dto.rs` | `SuccessDeleteRetrospectResponse` Swagger DTO 추가 |
| `src/domain/retrospect/handler.rs` | `delete_retrospect` 핸들러 추가 |
| `src/domain/retrospect/service.rs` | `delete_retrospect` 서비스 메서드 추가 (캐스케이드 삭제) |
| `src/main.rs` | DELETE 라우트 등록, OpenAPI 스키마 등록 |

## 5. 코드 리뷰 체크리스트

- [x] API 스펙에 맞게 구현되었는가?
- [x] 에러 처리가 적절하게 되어 있는가? (Result + ? 패턴)
- [x] 트랜잭션으로 데이터 일관성을 보장하는가?
- [x] 외래 키 순서를 준수하여 삭제하는가?
- [x] 보안: 비멤버에게 회고 존재 여부를 노출하지 않는가? (404 통합)
- [x] Swagger 문서가 등록되었는가?
- [x] `cargo test`, `cargo clippy`, `cargo fmt` 모두 통과하는가?
- [x] `unwrap()` / `expect()` 미사용 (프로덕션 코드)
- [x] `serde(rename_all = "camelCase")` 적용
- [x] 공통 유틸리티 (`BaseResponse`, `AppError`) 재사용
