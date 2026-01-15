# ===== Build Stage =====
FROM rust:1.84-alpine AS builder

# 빌드 의존성 설치
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

WORKDIR /app

# 의존성 캐싱을 위해 Cargo 파일만 먼저 복사
COPY Cargo.toml Cargo.lock ./

# 더미 소스로 의존성만 빌드 (캐싱 최적화)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 실제 소스 복사 및 빌드
COPY src ./src
RUN touch src/main.rs && \
    cargo build --release

# ===== Runtime Stage =====
FROM alpine:3.19

# CA 인증서 (HTTPS 요청용)
RUN apk add --no-cache ca-certificates

# 비root 사용자 생성
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

WORKDIR /app

# 바이너리 복사
COPY --from=builder /app/target/release/web3-server .

# 소유권 변경
RUN chown -R appuser:appgroup /app

USER appuser

# 환경 변수
ENV RUST_LOG=info
ENV SERVER_PORT=8080

EXPOSE 8080

# 헬스체크
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

CMD ["./web3-server"]
