# User Info API

## 개요
사용자 ID를 통해 사용자 정보를 조회하는 Mock API입니다. 
실제 데이터베이스가 아닌 하드코딩된 Mock 데이터를 반환합니다.

## 요청 (Request)
- Method: GET
- Endpoint: `/api/user/{id}`
- Path Parameters:
  - `id` (number): 조회할 사용자 ID
- Headers: 없음
- Body: 없음

## 응답 (Response)

### 성공 (200 OK)
```json
{
  "success": true,
  "data": {
    "id": 1,
    "name": "홍길동",
    "email": "hong@example.com",
    "role": "admin"
  },
  "message": "Success",
  "timestamp": "2026-01-15T12:44:31.038513+00:00"
}
```

### 실패 (404 Not Found)
```json
{
  "success": false,
  "error": "NOT_FOUND",
  "message": "User not found: id=999",
  "timestamp": "2026-01-15T12:44:42.725096+00:00"
}
```

## Mock 데이터
현재 다음 사용자 ID들이 조회 가능합니다:
- ID 1: 홍길동 (admin)
- ID 2: 김철수 (user)
- ID 3: 이영희 (user)

## 구현 세부사항

### 사용한 핸들러/미들웨어
- **핸들러**: `handlers/user.rs::get_user_info`
- **Path 파라미터 추출**: Axum의 `Path` extractor 사용
- **공통 유틸리티**:
  - `util::ApiResponse`: 성공 응답 구조체
  - `util::AppError::not_found`: 404 에러 생성 함수

### 적용한 비즈니스 로직
1. Path에서 user_id 추출
2. Mock 사용자 데이터 HashMap에서 해당 ID 조회
3. 존재하면 사용자 정보 반환, 없으면 404 에러

### 에러 처리 방식
- 존재하지 않는 사용자 ID: `AppError::not_found` 반환 (404)
- 잘못된 ID 형식 (문자열 등): Axum이 자동으로 400 Bad Request 처리

### 사용한 공통 유틸리티
- `ApiResponse<T>`: 통일된 성공 응답 구조
- `AppError`: 통일된 에러 처리 및 응답

### 주요 의사결정 사항
- **Mock 데이터 관리**: 함수 내에서 HashMap으로 관리 (실제 프로덕션에서는 DB 사용)
- **Path Parameter**: Axum의 `Path<u32>` 타입을 사용하여 타입 안전성 확보
- **클론 사용**: HashMap에서 조회한 데이터를 `.clone()`하여 반환 (소유권 이슈 해결)

## 테스트
수행한 API 인수 테스트 목록:
- [x] 정상 케이스 테스트 (ID 1 조회)
- [x] 존재하지 않는 사용자 테스트 (ID 999, 404 응답)

### 테스트 코드 위치
`tests/user_info_test.rs`

### 테스트 실행 방법
```bash
# 서버 실행 (별도 터미널)
cargo run

# 테스트 실행
cargo test user_info
```

### 수동 테스트
```bash
# 성공 케이스
curl http://127.0.0.1:3000/api/user/1
curl http://127.0.0.1:3000/api/user/2
curl http://127.0.0.1:3000/api/user/3

# 실패 케이스 (404)
curl http://127.0.0.1:3000/api/user/999
```

## 향후 개선 사항
- 실제 데이터베이스 연동
- 페이지네이션을 통한 전체 사용자 목록 조회 API
- 사용자 생성/수정/삭제 API
- 인증/인가 미들웨어 추가

