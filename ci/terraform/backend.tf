# =============================================================================
# Terraform Backend (S3 + DynamoDB)
# =============================================================================
#
# 🚀 사용 방법 (2단계 배포):
#
# [1단계] 로컬에서 S3/DynamoDB 먼저 생성:
#   - 아래 backend "s3" 블록은 주석 상태로 유지
#   - terraform init && terraform apply
#   - S3 버킷과 DynamoDB 테이블이 생성됨
#
# [2단계] S3 Backend 활성화:
#   - 아래 backend "s3" 블록 주석 해제
#   - terraform init -migrate-state (state를 S3로 이동)
#   - commit & push → 이후 GitHub Actions 정상 작동
#
# =============================================================================

# 1단계 완료 후 주석 해제하세요
terraform {
  backend "s3" {
    bucket         = "web-team-3-terraform-state"
    key            = "terraform.tfstate"
    region         = "ap-northeast-2"
    encrypt        = true
    dynamodb_table = "web-team-3-terraform-lock"
  }
}

# =============================================================================
# Backend 인프라 리소스
# =============================================================================

resource "aws_s3_bucket" "terraform_state" {
  bucket = "${var.project_name}-terraform-state"

  tags = {
    Name = "${var.project_name}-terraform-state"
  }
}

resource "aws_s3_bucket_versioning" "terraform_state" {
  bucket = aws_s3_bucket.terraform_state.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "terraform_state" {
  bucket = aws_s3_bucket.terraform_state.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "terraform_state" {
  bucket = aws_s3_bucket.terraform_state.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_dynamodb_table" "terraform_lock" {
  name         = "${var.project_name}-terraform-lock"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "LockID"

  attribute {
    name = "LockID"
    type = "S"
  }

  tags = {
    Name = "${var.project_name}-terraform-lock"
  }
}
