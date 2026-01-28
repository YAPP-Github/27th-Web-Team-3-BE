# =============================================================================
# Terraform Backend (S3 + DynamoDB)
# 팀 협업 시 상태 파일 공유를 위해 사용
# =============================================================================

# 주의: 이 백엔드를 사용하려면 먼저 S3 버킷과 DynamoDB 테이블을 수동으로 생성해야 합니다.
# 또는 아래 주석을 해제하지 않고 로컬 상태로 시작한 후, 나중에 마이그레이션할 수 있습니다.

# terraform {
#   backend "s3" {
#     bucket         = "web-team-3-terraform-state"
#     key            = "terraform.tfstate"
#     region         = "ap-northeast-2"
#     encrypt        = true
#     dynamodb_table = "web-team-3-terraform-lock"
#   }
# }

# =============================================================================
# 백엔드 인프라 (최초 1회만 생성)
# 아래 리소스들은 terraform state를 저장하기 위한 것입니다.
# =============================================================================

resource "aws_s3_bucket" "terraform_state" {
  bucket = "${var.project_name}-terraform-state"

  lifecycle {
    prevent_destroy = true
  }

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
