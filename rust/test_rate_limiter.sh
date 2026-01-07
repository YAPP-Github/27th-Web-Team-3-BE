#!/bin/bash

# RateLimiter 테스트 스크립트
# 동일한 이메일로 여러 번 요청하여 rate limiting 동작 확인

echo "=== RateLimiter 테스트 시작 ==="
echo ""

EMAIL="test@example.com"
URL="http://127.0.0.1:8080/api/auth/signup"

# 11번 요청 (제한은 10번)
for i in {1..11}
do
  echo "요청 #$i"
  RESPONSE=$(curl -s -w "\nHTTP_CODE:%{http_code}" -X POST "$URL" \
    -H "Content-Type: application/json" \
    -d "{
      \"email\": \"$EMAIL\",
      \"username\": \"테스트유저$i\",
      \"password\": \"password123!\",
      \"passwordConfirm\": \"password123!\"
    }")

  HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE" | cut -d: -f2)
  BODY=$(echo "$RESPONSE" | grep -v "HTTP_CODE")

  echo "HTTP Status: $HTTP_CODE"
  echo "Response: $BODY"
  echo ""

  # 짧은 대기
  sleep 0.1
done

echo "=== 테스트 완료 ==="

