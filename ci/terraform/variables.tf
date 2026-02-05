# =============================================================================
# General Variables
# =============================================================================

variable "project_name" {
  description = "프로젝트 이름"
  type        = string
  default     = "web-team-3"
}

variable "environment" {
  description = "환경 (dev, staging, prod)"
  type        = string
  default     = "dev"
}

variable "aws_region" {
  description = "AWS 리전"
  type        = string
  default     = "ap-northeast-2"
}

# =============================================================================
# VPC Variables
# =============================================================================

variable "vpc_cidr" {
  description = "VPC CIDR 블록"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "가용 영역 목록"
  type        = list(string)
  default     = ["ap-northeast-2a", "ap-northeast-2c"]
}

variable "public_subnet_cidrs" {
  description = "퍼블릭 서브넷 CIDR 블록"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24"]
}

variable "private_subnet_cidrs" {
  description = "프라이빗 서브넷 CIDR 블록"
  type        = list(string)
  default     = ["10.0.10.0/24", "10.0.20.0/24"]
}

# =============================================================================
# EC2 Variables
# =============================================================================

variable "ec2_instance_type" {
  description = "EC2 인스턴스 타입"
  type        = string
  default     = "t3.micro"
}

variable "ec2_key_name" {
  description = "EC2 SSH 키 페어 이름"
  type        = string
}

variable "ec2_ami_id" {
  description = "EC2 AMI ID (빈 값이면 최신 Ubuntu 24.04 LTS 사용)"
  type        = string
  default     = ""
}

# =============================================================================
# RDS Variables
# =============================================================================

variable "db_instance_class" {
  description = "RDS 인스턴스 클래스"
  type        = string
  default     = "db.t3.micro"
}

variable "db_engine" {
  description = "데이터베이스 엔진"
  type        = string
  default     = "mysql"
}

variable "db_engine_version" {
  description = "데이터베이스 엔진 버전"
  type        = string
  default     = "8.0"
}

variable "db_name" {
  description = "데이터베이스 이름"
  type        = string
  default     = "webteam3db"
}

variable "db_username" {
  description = "데이터베이스 마스터 사용자명"
  type        = string
  sensitive   = true
}

variable "db_password" {
  description = "데이터베이스 마스터 비밀번호"
  type        = string
  sensitive   = true
}

variable "db_allocated_storage" {
  description = "RDS 할당 스토리지 (GB)"
  type        = number
  default     = 20
}

variable "db_max_allocated_storage" {
  description = "RDS 최대 자동 확장 스토리지 (GB)"
  type        = number
  default     = 20  # 프리티어 한도에 맞춤 (자동 확장 비활성화)
}

variable "db_multi_az" {
  description = "Multi-AZ 배포 여부"
  type        = bool
  default     = false
}

variable "db_skip_final_snapshot" {
  description = "삭제 시 최종 스냅샷 건너뛰기"
  type        = bool
  default     = true
}

# =============================================================================
# Domain Variables
# =============================================================================

variable "domain_name" {
  description = "메인 도메인 이름 (Route53 호스팅 영역)"
  type        = string
  default     = "moaofficial.kr"
}

variable "api_subdomain" {
  description = "API 서브도메인"
  type        = string
  default     = "api"
}

variable "ssl_email" {
  description = "SSL 인증서 알림용 이메일"
  type        = string
  default     = "admin@moaofficial.kr"
}
