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
    apt-get install -y git build-essential pkg-config libssl-dev curl

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

    # Rust 설치 (ubuntu 사용자로)
    su - ubuntu -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'

    # 애플리케이션 디렉토리 생성
    mkdir -p /opt/app
    chown ubuntu:ubuntu /opt/app

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
