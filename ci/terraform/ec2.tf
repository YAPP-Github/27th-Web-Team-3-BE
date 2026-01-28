# =============================================================================
# AMI Data Source (Amazon Linux 2023)
# =============================================================================

data "aws_ami" "amazon_linux_2023" {
  most_recent = true
  owners      = ["amazon"]

  filter {
    name   = "name"
    values = ["al2023-ami-*-x86_64"]
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
  ami                    = var.ec2_ami_id != "" ? var.ec2_ami_id : data.aws_ami.amazon_linux_2023.id
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
    dnf update -y

    # 필수 패키지 설치
    dnf install -y git gcc openssl-devel

    # Docker 설치
    dnf install -y docker
    systemctl start docker
    systemctl enable docker
    usermod -aG docker ec2-user

    # Docker Compose 설치
    curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    chmod +x /usr/local/bin/docker-compose

    # Rust 설치 (ec2-user로)
    su - ec2-user -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'

    # 애플리케이션 디렉토리 생성
    mkdir -p /opt/app
    chown ec2-user:ec2-user /opt/app

    echo "EC2 초기화 완료"
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
