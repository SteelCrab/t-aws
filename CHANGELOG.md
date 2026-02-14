# Changelog

## [0.2.0] - 2026-02-14

### ✨ Features

- Added large-scale quality gate with scenario validation integrated into `tools/rust-coverage.sh`
- Added Codecov upload configuration for automated coverage reporting

### 🔧 Improvements

- Unified coverage pipeline by removing redundant scenario-check script and simplifying `tools` execution flow
- Updated CI coverage artifact path to match `cargo llvm-cov` output (`target/llvm-cov-target/lcov.info`)
- Added coverage threshold guidance and documentation updates in `tools/README.md` / `tools/README_KR.md`
- Added roadmap references in README and introduced `ROADMAP.md` / `ROADMAP_KR.md` as process documentation
- Added `.pre-commit-config.yaml` and documented pre-commit usage for `cargo fmt`, clippy, and tests

### 🐛 Fixes

- Normalized scenario/coverage tooling behavior to avoid stale references to removed scripts
- Ensured quality gate documentation and workflow align with actual executable set

---

## [0.1.1] - 2026-02-03

### ✨ Features

- **Self-Update Command**: `emd update` - Update to the latest version directly from GitHub releases
- **i18n Markdown Output**: AWS resource details now render with language-dependent labels (Korean/English)

### 🔧 Improvements

- Modular AWS CLI structure with separate modules for each service
- Base64 decoding using Rust crate instead of shell command for better security

### 🐛 Fixes

- Use `rustls` instead of `native-tls` for musl cross-compilation compatibility

---
---
"This patch focused on refactoring the overall EMD. And I'm very happy that automatic updates are now possible through the new update feature, emd update. 🎉"

---

## [0.1.0] - 2025-01-31

# 🚀 Initial Release

**EMD** - A TUI application for exploring AWS resources and generating Markdown documentation.

---

## ☁️ AWS Services

| Service | Description |
|:---|:---|
| EC2 | Instance details, IPs, security groups, tags |
| VPC | Subnets, IGW, NAT, route tables, EIPs + Mermaid diagram |
| Security Group | Inbound/outbound rules |
| Load Balancer | ALB/NLB/CLB, listeners, target groups |
| ECR | Repositories, tag mutability, encryption |

## 📋 Blueprint

- Combine resources from multiple regions into one document
- Auto-generated table of contents
- Reorder with `Shift+↑↓`

## 🌐 Multi-language

- Korean (default) / English
- Switch via Settings tab (`→` key)

## 🗺️ Regions

Seoul, Tokyo, Osaka, Singapore, Sydney, Mumbai, Virginia, Ohio, California, Oregon, Ireland, Frankfurt

---

## ⌨️ Shortcuts

| Key | Action |
|:---|:---|
| `↑↓` / `jk` | Navigate |
| `Enter` | Select |
| `Esc` | Back |
| `→` / `←` | Switch tabs |
| `r` | Refresh |
| `s` | Save |
| `q` | Quit |
