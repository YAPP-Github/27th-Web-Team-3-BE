# =============================================================================
# VPC Outputs
# =============================================================================

output "vpc_id" {
  description = "VPC ID"
  value       = aws_vpc.main.id
}

output "public_subnet_ids" {
  description = "퍼블릭 서브넷 ID 목록"
  value       = aws_subnet.public[*].id
}

output "private_subnet_ids" {
  description = "프라이빗 서브넷 ID 목록"
  value       = aws_subnet.private[*].id
}

# =============================================================================
# EC2 Outputs
# =============================================================================

output "ec2_instance_id" {
  description = "EC2 인스턴스 ID"
  value       = aws_instance.app.id
}

output "ec2_public_ip" {
  description = "EC2 퍼블릭 IP (Elastic IP)"
  value       = aws_eip.app.public_ip
}

output "ec2_public_dns" {
  description = "EC2 퍼블릭 DNS"
  value       = aws_eip.app.public_dns
}

# =============================================================================
# RDS Outputs
# =============================================================================

output "rds_endpoint" {
  description = "RDS 엔드포인트"
  value       = aws_db_instance.main.endpoint
}

output "rds_hostname" {
  description = "RDS 호스트명"
  value       = aws_db_instance.main.address
}

output "rds_port" {
  description = "RDS 포트"
  value       = aws_db_instance.main.port
}

output "rds_database_name" {
  description = "RDS 데이터베이스 이름"
  value       = aws_db_instance.main.db_name
}

# =============================================================================
# Connection Info
# =============================================================================

output "ssh_connection" {
  description = "SSH 접속 명령어"
  value       = "ssh -i <your-key.pem> ubuntu@${aws_eip.app.public_ip}"
}

output "database_connection_string" {
  description = "데이터베이스 연결 문자열 (비밀번호 제외)"
  value       = "mysql://${var.db_username}:<password>@${aws_db_instance.main.address}:${aws_db_instance.main.port}/${aws_db_instance.main.db_name}"
  sensitive   = true
}

# =============================================================================
# Route53 & Domain Outputs
# =============================================================================

output "api_domain" {
  description = "API 도메인 (HTTPS)"
  value       = "https://${var.api_subdomain}.${var.domain_name}"
}

output "api_domain_name" {
  description = "API 도메인 이름"
  value       = "${var.api_subdomain}.${var.domain_name}"
}

output "ssl_setup_command" {
  description = "SSL 인증서 발급 명령어 (DNS 전파 후 실행)"
  value       = "sudo certbot --nginx -d ${var.api_subdomain}.${var.domain_name}"
}
