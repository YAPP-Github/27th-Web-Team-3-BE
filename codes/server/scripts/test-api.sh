#!/bin/bash
# API 테스트 스크립트
# 사용법: ./scripts/test-api.sh

set -e

BASE_URL="http://localhost:8080"
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================"
echo "회고 생성 API 테스트"
echo "========================================"

# 1. Health Check
echo -e "\n${YELLOW}[1] Health Check${NC}"
HEALTH=$(curl -s "$BASE_URL/health")
echo "$HEALTH" | jq .
if echo "$HEALTH" | jq -e '.isSuccess == true' > /dev/null; then
    echo -e "${GREEN}✓ Health check passed${NC}"
else
    echo -e "${RED}✗ Health check failed${NC}"
    exit 1
fi

# 2. 로그인하여 토큰 획득
echo -e "\n${YELLOW}[2] 이메일 로그인 (테스트 토큰 획득)${NC}"
LOGIN_RESP=$(curl -s -X POST "$BASE_URL/api/auth/login/email" \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com"}')
echo "$LOGIN_RESP" | jq .

TOKEN=$(echo "$LOGIN_RESP" | jq -r '.result.accessToken // empty')
if [ -z "$TOKEN" ]; then
    echo -e "${RED}✗ Failed to get token${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Token acquired${NC}"

# 3. 회고 생성 - 정상 케이스
echo -e "\n${YELLOW}[3] 회고 생성 (정상 케이스)${NC}"
FUTURE_DATE=$(date -v+7d +%Y-%m-%d 2>/dev/null) || FUTURE_DATE=$(date -d "+7 days" +%Y-%m-%d)
CREATE_RESP=$(curl -s -X POST "$BASE_URL/api/v1/retrospects" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"teamId\": 1,
    \"projectName\": \"테스트 프로젝트\",
    \"retrospectDate\": \"$FUTURE_DATE\",
    \"retrospectMethod\": \"KPT\",
    \"referenceUrls\": [\"https://github.com/example\"]
  }")
echo "$CREATE_RESP" | jq .

if echo "$CREATE_RESP" | jq -e '.isSuccess == true' > /dev/null; then
    RETRO_ID=$(echo "$CREATE_RESP" | jq '.result.retrospectId')
    echo -e "${GREEN}✓ Retrospect created (ID: $RETRO_ID)${NC}"
else
    echo -e "${RED}✗ Create failed: $(echo "$CREATE_RESP" | jq -r '.message')${NC}"
    exit 1
fi

# 4. 에러 케이스 - 인증 없음
echo -e "\n${YELLOW}[4] 에러 테스트: 인증 없음${NC}"
NO_AUTH_RESP=$(curl -s -X POST "$BASE_URL/api/v1/retrospects" \
  -H "Content-Type: application/json" \
  -d "{\"teamId\": 1, \"projectName\": \"Test\", \"retrospectDate\": \"$FUTURE_DATE\", \"retrospectMethod\": \"KPT\"}")
echo "$NO_AUTH_RESP" | jq .
if echo "$NO_AUTH_RESP" | jq -e '.code == "AUTH4001"' > /dev/null; then
    echo -e "${GREEN}✓ Correctly returned AUTH4001${NC}"
else
    echo -e "${RED}✗ Expected AUTH4001 but got: $(echo "$NO_AUTH_RESP" | jq -r '.code')${NC}"
    exit 1
fi

# 5. 에러 케이스 - 프로젝트 이름 초과
echo -e "\n${YELLOW}[5] 에러 테스트: 프로젝트 이름 21자 초과${NC}"
LONG_NAME_RESP=$(curl -s -X POST "$BASE_URL/api/v1/retrospects" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"teamId\": 1,
    \"projectName\": \"123456789012345678901\",
    \"retrospectDate\": \"$FUTURE_DATE\",
    \"retrospectMethod\": \"KPT\"
  }")
echo "$LONG_NAME_RESP" | jq .
if echo "$LONG_NAME_RESP" | jq -e '.code == "RETRO4001"' > /dev/null; then
    echo -e "${GREEN}✓ Correctly returned RETRO4001${NC}"
else
    echo -e "${RED}✗ Expected RETRO4001 but got: $(echo "$LONG_NAME_RESP" | jq -r '.code')${NC}"
    exit 1
fi

# 6. 에러 케이스 - 유효하지 않은 회고 방식
echo -e "\n${YELLOW}[6] 에러 테스트: 유효하지 않은 회고 방식${NC}"
INVALID_METHOD_RESP=$(curl -s -X POST "$BASE_URL/api/v1/retrospects" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"teamId\": 1,
    \"projectName\": \"Test\",
    \"retrospectDate\": \"$FUTURE_DATE\",
    \"retrospectMethod\": \"INVALID_METHOD\"
  }")
echo "$INVALID_METHOD_RESP" | jq .
if echo "$INVALID_METHOD_RESP" | jq -e '.code == "RETRO4005"' > /dev/null; then
    echo -e "${GREEN}✓ Correctly returned RETRO4005${NC}"
else
    echo -e "${RED}✗ Expected RETRO4005 but got: $(echo "$INVALID_METHOD_RESP" | jq -r '.code')${NC}"
    exit 1
fi

echo -e "\n========================================"
echo -e "${GREEN}테스트 완료!${NC}"
echo "========================================"
