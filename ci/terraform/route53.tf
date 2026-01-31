# =============================================================================
# Route53 DNS Configuration
# =============================================================================

# 기존 호스팅 영역 참조 (AWS 콘솔에서 생성됨)
data "aws_route53_zone" "main" {
  name         = var.domain_name
  private_zone = false
}

# API 서브도메인 A 레코드 (api.moaofficial.kr -> EC2 Elastic IP)
resource "aws_route53_record" "api" {
  zone_id = data.aws_route53_zone.main.zone_id
  name    = "api.${var.domain_name}"
  type    = "A"
  ttl     = 300
  records = [aws_eip.app.public_ip]
}
