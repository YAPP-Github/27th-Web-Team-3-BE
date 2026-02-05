# Terraform Infrastructure

Web Team 3 백엔드 인프라를 AWS에 프로비저닝하는 Terraform 구성입니다.

## 아키텍처

```
┌─────────────────────────────────────────────────────────────────┐
│                            VPC                                   │
│  ┌─────────────────────────┐  ┌─────────────────────────────┐  │
│  │    Public Subnet (AZ-a) │  │    Public Subnet (AZ-c)     │  │
│  │  ┌───────────────────┐  │  │                             │  │
│  │  │   EC2 (App)       │  │  │                             │  │
│  │  │   + Elastic IP    │  │  │                             │  │
│  │  └───────────────────┘  │  │                             │  │
│  └─────────────────────────┘  └─────────────────────────────┘  │
│                                                                  │
│  ┌─────────────────────────┐  ┌─────────────────────────────┐  │
│  │   Private Subnet (AZ-a) │  │   Private Subnet (AZ-c)     │  │
│  │  ┌───────────────────┐  │  │                             │  │
│  │  │   RDS (MySQL)     │  │  │   (Multi-AZ Standby)        │  │
│  │  └───────────────────┘  │  │                             │  │
│  └─────────────────────────┘  └─────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## 리소스

| 리소스 | 설명 |
|--------|------|
| VPC | 10.0.0.0/16 CIDR 블록 |
| Public Subnets | EC2 인스턴스용 (인터넷 접근 가능) |
| Private Subnets | RDS용 (인터넷 접근 불가) |
| EC2 | Ubuntu 24.04 LTS, t3.micro, Nginx + Certbot |
| RDS | MySQL 8.0, db.t3.micro |
| Route53 | api.moaofficial.kr A 레코드 |
| Security Groups | EC2, RDS 각각 별도 구성 |

## 사전 요구사항

1. AWS CLI 설치 및 설정
2. Terraform 1.0+ 설치
3. AWS 키 페어 생성

```bash
# AWS CLI 설정
aws configure

# Terraform 설치 (macOS)
brew install terraform
```

## 사용법

### 1. 변수 파일 설정

```bash
cd ci/terraform
cp terraform.tfvars.example terraform.tfvars
```

`terraform.tfvars` 파일을 열고 값을 수정하세요:
- `ec2_key_name`: AWS에서 생성한 키 페어 이름
- `db_username`: 데이터베이스 관리자 계정
- `db_password`: 안전한 비밀번호

### 2. Terraform 초기화

```bash
terraform init
```

### 3. 실행 계획 확인

```bash
terraform plan
```

### 4. 인프라 생성

```bash
terraform apply
```

### 5. 출력 확인

```bash
terraform output
```

## 접속 방법

### EC2 SSH 접속

```bash
ssh -i <your-key.pem> ubuntu@<ec2_public_ip>
```

### RDS 접속 (EC2 경유)

EC2에 접속 후:
```bash
mysql -h <rds_hostname> -u <db_username> -p <db_name>
```

## HTTPS 설정 (SSL 인증서)

EC2에는 Nginx와 Certbot이 사전 설치됩니다. SSL 인증서 발급은 DNS 전파 후 수동으로 실행합니다.

### 1. DNS 전파 확인

```bash
# EC2에서 실행
dig +short api.moaofficial.kr
curl -s http://checkip.amazonaws.com

# 두 IP가 일치해야 함
```

### 2. SSL 인증서 발급

```bash
# EC2에서 실행
sudo certbot --nginx -d api.moaofficial.kr
```

### 3. 자동 갱신 확인

```bash
sudo certbot renew --dry-run
```

### 참고: 수동 설정 스크립트

`ci/scripts/setup-ssl.sh` 스크립트를 EC2에 복사하여 사용할 수도 있습니다:
```bash
scp -i <your-key.pem> ci/scripts/setup-ssl.sh ubuntu@<ec2_ip>:~/
ssh -i <your-key.pem> ubuntu@<ec2_ip> 'sudo ~/setup-ssl.sh'
```

## 인프라 삭제

```bash
terraform destroy
```

## 환경별 구성

### Development (기본값)
- EC2: t3.micro
- RDS: db.t3.micro, 단일 AZ
- 삭제 보호: 비활성화

### Production (권장 설정)
```hcl
environment              = "prod"
ec2_instance_type        = "t3.small"
db_instance_class        = "db.t3.small"
db_multi_az              = true
db_skip_final_snapshot   = false
```

## 보안 주의사항

- `terraform.tfvars` 파일은 절대 Git에 커밋하지 마세요
- 프로덕션에서는 AWS Secrets Manager 사용 권장
- SSH 접근은 필요한 IP로 제한하세요

## 비용 참고

Free Tier 범위 내 예상 리소스:
- EC2 t3.micro: 월 750시간 무료 (1년)
- RDS db.t3.micro: 월 750시간 무료 (1년)
- 스토리지: 20GB까지 무료

Free Tier 초과 시 예상 월 비용:
- EC2 t3.micro: ~$8-10
- RDS db.t3.micro: ~$15-20
- Elastic IP: 사용 중이면 무료
