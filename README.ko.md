[🇺🇸 English](README.md) | [🇰🇷 한국어](README.ko.md)

# AWS CLI 설치 및 도구

Rust로 만든 크로스 플랫폼 AWS CLI v2 설치 도구 및 빠른 참조 도구입니다.

## 기능

- 🖥️ **크로스 플랫폼**: macOS, Windows, Linux (x86_64, arm64)
- 📥 **설치/삭제**: 진행률 바와 함께 AWS CLI v2 설치
- 📖 **치트시트**: S3, EC2, IAM 명령어 빠른 참조
- 🌏 **실시간 리소스**: EC2 인스턴스 및 S3 버킷 실시간 조회

## 설치

[Releases](../../releases)에서 다운로드하거나 소스에서 빌드:

```bash
cargo build --release
./target/release/t-aws
```

## 사용법

### TUI 모드 (인터랙티브)
```bash
./t-aws
# Enter 누름 → [1] 설치 또는 [2] 삭제 선택
```

### CLI 모드 (직접 실행)
```bash
./t-aws -i              # AWS CLI 설치
./t-aws -u              # AWS CLI 삭제
```

### 치트시트
```bash
./t-aws s3              # S3 명령어 참조
./t-aws ec2             # EC2 명령어 참조
./t-aws iam             # IAM 명령어 참조
```

### 실시간 AWS 리소스
```bash
./t-aws resources                    # 현재 리전
./t-aws resources -r ap-northeast-2  # 특정 리전
```

> ⚠️ `resources` 명령어는 AWS 자격 증명이 필요합니다 (`aws configure`)

## 지원 플랫폼

| OS | 아키텍처 |
|----|----------|
| macOS | x86_64, arm64 |
| Windows | x86_64 |
| Linux | x86_64, arm64 |

## 라이선스

MIT
