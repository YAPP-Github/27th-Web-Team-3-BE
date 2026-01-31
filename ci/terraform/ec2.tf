# =============================================================================
# AMI Data Source (Ubuntu 24.04 LTS)
# =============================================================================

data "aws_ami" "ubuntu_2404" {
  most_recent = true
  owners      = ["099720109477"] # Canonical

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-amd64-server-*"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

# =============================================================================
# EC2 Instance
# =============================================================================

resource "aws_instance" "app" {
  ami                    = var.ec2_ami_id != "" ? var.ec2_ami_id : data.aws_ami.ubuntu_2404.id
  instance_type          = var.ec2_instance_type
  key_name               = var.ec2_key_name
  subnet_id              = aws_subnet.public[0].id
  vpc_security_group_ids = [aws_security_group.ec2.id]

  root_block_device {
    volume_type           = "gp3"
    volume_size           = 20
    delete_on_termination = true
    encrypted             = true
  }

  user_data = <<-EOF
    #!/bin/bash
    set -e

    # 시스템 업데이트
    apt-get update && apt-get upgrade -y

    # 필수 패키지 설치
    apt-get install -y git build-essential pkg-config libssl-dev curl dnsutils

    # Docker 설치
    apt-get install -y ca-certificates curl gnupg
    install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    chmod a+r /etc/apt/keyrings/docker.gpg
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
    apt-get update
    apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
    systemctl start docker
    systemctl enable docker
    usermod -aG docker ubuntu

    # Nginx 설치
    apt-get install -y nginx
    systemctl start nginx
    systemctl enable nginx

    # Certbot 설치 (snap 기반 - Ubuntu 24.04 권장)
    snap install core
    snap refresh core
    snap install --classic certbot
    ln -sf /snap/bin/certbot /usr/bin/certbot

    # Nginx 설정 배치 (HTTP only - SSL은 수동 설정)
    cat <<'NGINX_CONF' > /etc/nginx/sites-available/api.conf
server {
    listen 80;
    listen [::]:80;
    server_name ${var.api_subdomain}.${var.domain_name};

    # Certbot webroot 인증용
    location /.well-known/acme-challenge/ {
        root /var/www/html;
    }

    # 리버스 프록시 (SSL 설정 전)
    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Connection "";
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
}
NGINX_CONF

    # 사이트 활성화
    ln -sf /etc/nginx/sites-available/api.conf /etc/nginx/sites-enabled/
    rm -f /etc/nginx/sites-enabled/default

    # Nginx 설정 테스트 및 재시작
    nginx -t && systemctl reload nginx

    # Rust 설치 (ubuntu 사용자로)
    su - ubuntu -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'

    # 애플리케이션 디렉토리 생성
    mkdir -p /opt/app
    chown ubuntu:ubuntu /opt/app

    echo "=============================================="
    echo "EC2 초기화 완료"
    echo ""
    echo "SSL 인증서 발급 (DNS 전파 후 수동 실행):"
    echo "  sudo certbot --nginx -d ${var.api_subdomain}.${var.domain_name}"
    echo "=============================================="
  EOF

  tags = {
    Name = "${var.project_name}-${var.environment}-app-server"
  }

  lifecycle {
    ignore_changes = [ami]
  }
}

# =============================================================================
# Elastic IP
# =============================================================================

resource "aws_eip" "app" {
  instance = aws_instance.app.id
  domain   = "vpc"

  tags = {
    Name = "${var.project_name}-${var.environment}-app-eip"
  }
}
