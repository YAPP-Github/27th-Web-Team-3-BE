#!/bin/bash

# 회고 작성 가이드 API 테스트 스크립트

echo "================================"
echo "회고 작성 가이드 API 테스트"
echo "================================"

# 1. 정상 케이스 테스트
echo -e "\n[테스트 1] 정상 요청"
curl -X POST http://127.0.0.1:8080/api/ai/retrospective/guide \
  -H "Content-Type: application/json" \
  -d '{
    "currentContent": "오늘 프로젝트를 진행하면서 어려움이 있었습니다",
    "secretKey": "test_secret_key_123"
  }' | jq .

# 2. content 누락 케이스
echo -e "\n[테스트 2] content 누락 (COMMON400 예상)"
curl -X POST http://127.0.0.1:8080/api/ai/retrospective/guide \
  -H "Content-Type: application/json" \
  -d '{
    "currentContent": "",
    "secretKey": "test_secret_key_123"
  }' | jq .

# 3. secretKey 누락 케이스
echo -e "\n[테스트 3] secretKey 누락 (COMMON400 예상)"
curl -X POST http://127.0.0.1:8080/api/ai/retrospective/guide \
  -H "Content-Type: application/json" \
  -d '{
    "currentContent": "테스트 내용",
    "secretKey": ""
  }' | jq .

# 4. 잘못된 secretKey 케이스
echo -e "\n[테스트 4] 잘못된 secretKey (AI_001 예상)"
curl -X POST http://127.0.0.1:8080/api/ai/retrospective/guide \
  -H "Content-Type: application/json" \
  -d '{
    "currentContent": "테스트 내용",
    "secretKey": "wrong_key"
  }' | jq .

echo -e "\n================================"
echo "회고 다듬기 API 테스트"
echo "================================"

# 5. 회고 다듬기 - 정중체
echo -e "\n[테스트 5] 회고 다듬기 - 정중체"
curl -X POST http://127.0.0.1:8080/api/ai/retrospective/refine \
  -H "Content-Type: application/json" \
  -d '{
    "content": "오늘 일 존나 힘들었음 ㅋㅋ",
    "toneStyle": "POLITE",
    "secretKey": "test_secret_key_123"
  }' | jq .

# 6. 회고 다듬기 - 상냥체
echo -e "\n[테스트 6] 회고 다듬기 - 상냥체"
curl -X POST http://127.0.0.1:8080/api/ai/retrospective/refine \
  -H "Content-Type: application/json" \
  -d '{
    "content": "오늘 일 존나 힘들었음 ㅋㅋ",
    "toneStyle": "KIND",
    "secretKey": "test_secret_key_123"
  }' | jq .

echo -e "\n================================"
echo "테스트 완료"
echo "================================"

