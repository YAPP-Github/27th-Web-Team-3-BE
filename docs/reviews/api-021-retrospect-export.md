# API-021: 회고 내보내기 API 구현 리뷰

## 개요
특정 회고 세션의 전체 내용(인사이트, 팀원별 답변 등)을 요약하여 PDF 파일로 생성하고 다운로드하는 API입니다.

## 엔드포인트
- **Method**: `GET`
- **Path**: `/api/v1/retrospects/{retrospectId}/export`
- **인증**: Bearer Token 필수

## 요청
| 파라미터 | 타입 | 위치 | 필수 | 설명 |
|----------|------|------|------|------|
| retrospectId | integer(int64) | Path | Y | 내보낼 회고의 고유 식별자 (1 이상) |

## 응답

### 성공 (200)
- **Content-Type**: `application/pdf; charset=utf-8`
- **Content-Disposition**: `attachment; filename="retrospect_report_{id}_{timestamp}.pdf"`
- **Cache-Control**: `no-cache, no-store, must-revalidate`
- **Body**: PDF 바이너리 데이터

### 에러 응답
| 코드 | HTTP | 설명 |
|------|------|------|
| COMMON400 | 400 | retrospectId 유효성 오류 (0 이하) |
| AUTH4001 | 401 | 인증 실패 (토큰 없음/만료) |
| RETRO4031 | 403 | 접근 권한 없음 (회고방 멤버가 아님) |
| RETRO4041 | 404 | 존재하지 않는 회고 |
| COMMON500 | 500 | PDF 생성 실패 / 서버 내부 오류 |

## 구현 세부사항

### 변경 파일
| 파일 | 변경 내용 |
|------|----------|
| `Cargo.toml` | `genpdf = "0.2"` PDF 생성 라이브러리 추가 |
| `src/utils/error.rs` | `PdfGenerationFailed` 에러 변형 추가 (COMMON500) |
| `src/domain/retrospect/handler.rs` | `export_retrospect` 핸들러 추가 (바이너리 응답) |
| `src/domain/retrospect/service.rs` | `export_retrospect`, `generate_pdf`, `retrospect_method_display` 메서드 추가 |
| `src/main.rs` | 라우트 등록 및 Swagger 경로 추가 |
| `tests/retrospect_export_test.rs` | 통합 테스트 11개 추가 |

### 비즈니스 로직 흐름
1. **인증 확인**: Bearer 토큰에서 사용자 ID 추출
2. **retrospectId 검증**: 1 이상의 양수 확인
3. **회고 조회 및 접근 제어**: `find_retrospect_for_member`로 회고 존재 및 회고방 멤버십 확인 (정보 누출 방지)
4. **회고방 이름 조회**: `team` 테이블에서 회고방명 조회
5. **참여 멤버 조회**: `member_retro` + `member` 테이블 조인으로 참여자 목록 및 닉네임 조회
6. **질문/답변 조회**: `response` 테이블에서 해당 회고의 모든 질문/답변 조회
7. **PDF 생성**: `genpdf` 라이브러리로 PDF 문서 생성
8. **바이너리 응답**: Content-Type, Content-Disposition, Cache-Control 헤더와 함께 PDF 바이트 반환

### PDF 문서 구성
1. **제목**: `{프로젝트명} - Retrospect Report`
2. **기본 정보**: 회고방명, 날짜/시간, 회고 방식, 참여 멤버 목록
3. **인사이트**: AI 분석 결과 (있는 경우)
4. **질문/답변**: 중복 제거된 질문별 답변 목록
5. **개인 인사이트**: 멤버별 개인 인사이트 (있는 경우)

### 폰트 설정
- `PDF_FONT_DIR` 환경변수로 폰트 디렉토리 설정 (기본값: `./fonts`)
- `PDF_FONT_FAMILY` 환경변수로 폰트 패밀리 설정 (기본값: `NanumGothic`)
- 한글 PDF 생성을 위해 CJK 지원 폰트 파일(TTF) 배포 필요

### 핸들러 특이사항
- 기존 핸들러들은 `Result<Json<BaseResponse<T>>, AppError>`를 반환하지만, 이 핸들러는 바이너리 응답이므로 `Result<impl IntoResponse, AppError>`를 반환
- Axum의 튜플 응답 패턴 `(headers, body)` 활용

## 테스트

### 서비스 단위 테스트 (service.rs) - 5개
- `should_display_kpt_as_kpt` - KPT 방식 표시명
- `should_display_four_l_as_4l` - 4L 방식 표시명
- `should_display_five_f_as_5f` - 5F 방식 표시명
- `should_display_pmi_as_pmi` - PMI 방식 표시명
- `should_display_free_as_free` - Free 방식 표시명

### 통합 테스트 (retrospect_export_test.rs) - 11개

#### 인증
- `api021_should_return_401_when_authorization_header_missing` - 인증 헤더 없음
- `api021_should_return_401_when_authorization_header_format_invalid` - 잘못된 인증 형식

#### Path 파라미터 검증
- `api021_should_return_400_when_retrospect_id_is_zero` - retrospectId가 0
- `api021_should_return_400_when_retrospect_id_is_negative` - retrospectId가 음수

#### 비즈니스 에러
- `api021_should_return_404_when_retrospect_not_found` - 존재하지 않는 회고
- `api021_should_return_403_when_user_is_not_team_member` - 접근 권한 없음
- `api021_should_return_500_when_pdf_generation_fails` - PDF 생성 실패

#### 성공 케이스
- `api021_should_return_200_with_pdf_binary` - PDF 바이너리 반환
- `api021_should_return_content_type_application_pdf` - Content-Type 검증
- `api021_should_return_content_disposition_with_filename` - Content-Disposition 파일명 검증
- `api021_should_return_cache_control_no_cache` - Cache-Control 검증

## 코드 리뷰 체크리스트
- [x] TDD 원칙을 따라 테스트 코드가 작성되었는가?
- [x] 모든 테스트가 통과하는가? (122 unit + 48 integration = 170 tests)
- [x] API 문서가 `docs/reviews/` 디렉토리에 작성되었는가?
- [x] 공통 유틸리티를 재사용했는가? (AppError, AuthUser, find_retrospect_for_member)
- [x] 에러 처리가 적절하게 되어 있는가? (COMMON400, AUTH4001, RETRO4031, RETRO4041, COMMON500)
- [x] 코드가 Rust 컨벤션을 따르는가? (cargo fmt, cargo clippy -- -D warnings)
- [x] 불필요한 의존성이 추가되지 않았는가? (genpdf만 추가 - PDF 생성에 필수)

---

## 버그 수정 이력

### [2026-01-30] PDF 내보내기 500 에러 수정 (Issue #47)

#### 문제 원인
프로덕션 환경에서 PDF 내보내기 API 호출 시 500 Internal Server Error 발생.
주요 원인은 배포 환경에서 폰트 파일 누락 및 환경변수 미설정.

#### 수정 내용

| 파일 | 변경 내용 |
|------|----------|
| `.env.example` | `PDF_FONT_DIR`, `PDF_FONT_FAMILY` 환경변수 추가 |
| `.github/workflows/deploy.yml` | 폰트 파일 아티팩트 업로드 및 EC2 배포 단계 추가 |
| `src/main.rs` | 서버 시작 시 폰트 파일 존재 여부 검증 로직 추가 |
| `src/domain/retrospect/service.rs` | PDF 생성 시 상세 에러 로깅 추가 |

#### 수정 상세

1. **환경변수 문서화** (`.env.example`)
   ```bash
   # PDF Generation
   PDF_FONT_DIR=./fonts
   PDF_FONT_FAMILY=NanumGothic
   ```

2. **배포 스크립트 개선** (`.github/workflows/deploy.yml`)
   - 빌드 단계: `server-fonts` 아티팩트 업로드 추가
   - 배포 단계: 폰트 아티팩트 다운로드 및 EC2로 SCP 복사 추가

3. **서버 시작 시 폰트 검증** (`src/main.rs`)
   - `validate_pdf_fonts()` 함수 추가
   - 폰트 디렉토리 및 Regular 폰트 파일 존재 여부 확인
   - 문제 발견 시 `warn!` 로그 출력 (서버 시작 차단하지 않음)

4. **에러 로깅 개선** (`src/domain/retrospect/service.rs`)
   - PDF 생성 시작/완료 로그 추가
   - 폰트 로딩 성공/실패 상세 로그 추가
   - 에러 발생 시 `error!` 레벨 로그로 원인 기록

#### 테스트 결과
- 모든 기존 테스트 통과 (122 unit + 48 integration tests)
- `cargo fmt` 적용 완료
- `cargo clippy -- -D warnings` 경고 없음

#### 배포 후 확인사항
- [ ] EC2에 `/opt/app/fonts/` 디렉토리 존재 확인
- [ ] NanumGothic 폰트 파일 4개 존재 확인
- [ ] 서버 로그에서 "PDF 폰트 검증 완료" 메시지 확인
- [ ] PDF 내보내기 API 정상 동작 확인

### [2026-01-30] 배포 스크립트 PDF 환경변수 자동 설정 추가

#### 문제 원인
`secrets.ENV_FILE`에 PDF 환경변수가 포함되지 않을 경우, 배포 환경에서 기본값이 사용됨.
상대 경로 `./fonts`는 `WorkingDirectory=/opt/app`에서 정상 작동하지만, 명시적 설정이 더 안전함.

#### 수정 내용

| 파일 | 변경 내용 |
|------|----------|
| `.github/workflows/deploy.yml` | `.env` 파일 생성 시 PDF 환경변수 자동 추가 |

```yaml
# .env 파일 생성 후 PDF 환경변수 자동 추가
grep -q "PDF_FONT_DIR" /opt/app/.env || echo "PDF_FONT_DIR=/opt/app/fonts" >> /opt/app/.env
grep -q "PDF_FONT_FAMILY" /opt/app/.env || echo "PDF_FONT_FAMILY=NanumGothic" >> /opt/app/.env
```

#### 효과
- `secrets.ENV_FILE`에 PDF 환경변수가 없어도 배포 시 자동 설정
- 절대 경로 `/opt/app/fonts` 사용으로 경로 해석 오류 방지
- 기존 설정이 있으면 덮어쓰지 않음 (`grep -q`로 체크)
