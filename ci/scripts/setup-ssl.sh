#!/bin/bash
# =============================================================================
# SSL 인증서 발급 스크립트 (수동 실행)
# =============================================================================
#
# 사용법:
#   sudo ./setup-ssl.sh
#
# 사전 조건:
#   1. DNS 레코드가 EC2 IP를 가리키고 있어야 함
#   2. Nginx가 실행 중이어야 함
#   3. 80 포트가 열려있어야 함
#
# =============================================================================

set -e

DOMAIN="api.moaofficial.kr"
EMAIL="admin@moaofficial.kr"

echo "=== SSL 인증서 발급 스크립트 ==="
echo ""

# 1. DNS 전파 확인
echo "[1/4] DNS 전파 확인 중..."
RESOLVED_IP=$(dig +short "$DOMAIN" | head -1)
CURRENT_IP=$(curl -s http://checkip.amazonaws.com)

if [ "$RESOLVED_IP" != "$CURRENT_IP" ]; then
    echo "⚠️  경고: DNS가 아직 전파되지 않았습니다."
    echo "    도메인 IP: $RESOLVED_IP"
    echo "    현재 서버 IP: $CURRENT_IP"
    echo ""
    read -p "계속 진행하시겠습니까? (y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "취소되었습니다."
        exit 1
    fi
else
    echo "✅ DNS 정상 (IP: $RESOLVED_IP)"
fi

# 2. Nginx 상태 확인
echo ""
echo "[2/4] Nginx 상태 확인 중..."
if ! systemctl is-active --quiet nginx; then
    echo "Nginx 시작 중..."
    sudo systemctl start nginx
fi
echo "✅ Nginx 실행 중"

# 3. Certbot으로 인증서 발급
echo ""
echo "[3/4] Certbot으로 인증서 발급 중..."
sudo certbot --nginx -d "$DOMAIN" --non-interactive --agree-tos --email "$EMAIL"

# 4. Nginx 설정 테스트 및 재시작
echo ""
echo "[4/4] Nginx 설정 확인 및 재시작..."
sudo nginx -t
sudo systemctl reload nginx

echo ""
echo "=== 완료 ==="
echo "✅ SSL 인증서가 성공적으로 발급되었습니다."
echo "✅ https://$DOMAIN 으로 접속 가능합니다."
echo ""
echo "인증서 자동 갱신 확인:"
echo "  sudo certbot renew --dry-run"
