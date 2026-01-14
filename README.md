[🇺🇸 English](README.md) | [🇰🇷 한국어](README.ko.md)

# AWS CLI Installer & Tools

A cross-platform AWS CLI v2 installer and quick reference tool, built with Rust.

## Features

- 🖥️ **Cross-platform**: macOS, Windows, Linux (x86_64, arm64)
- 📥 **Install/Uninstall**: AWS CLI v2 with progress bar
- 📖 **Cheatsheets**: Quick reference for S3, EC2, IAM commands
- 🌏 **Live Resources**: View EC2 instances & S3 buckets in real-time

## Installation

Download from [Releases](../../releases) or build from source:

```bash
cargo build --release
./target/release/t-aws
```

## Usage

### TUI Mode (Interactive)
```bash
./t-aws
# Press Enter → Select [1] Install or [2] Uninstall
```

### CLI Mode (Direct)
```bash
./t-aws -i              # Install AWS CLI
./t-aws -u              # Uninstall AWS CLI
```

### Cheatsheets
```bash
./t-aws s3              # S3 command reference
./t-aws ec2             # EC2 command reference
./t-aws iam             # IAM command reference
```

### Live AWS Resources
```bash
./t-aws resources                    # Current region
./t-aws resources -r ap-northeast-2  # Specific region
```

> ⚠️ `resources` command requires AWS credentials (`aws configure`)

## Supported Platforms

| OS | Architecture |
|----|-------------|
| macOS | x86_64, arm64 |
| Windows | x86_64 |
| Linux | x86_64, arm64 |

## License

MIT
