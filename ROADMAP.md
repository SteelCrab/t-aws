# EMD Roadmap

> Development roadmap for AWS resource exploration and documentation tool

## Current Status

- **Base Branch**: `main`
- **Working Branches**: `feat/large-scale-tests-coverage-85`, `feat/aws-sdk-foundation`, `feat/aws-sdk-core-services`, `feat/aws-sdk-remaining-services`
- **Latest Version**: v0.1.1
- Core AWS services supported (EC2, VPC, Security Groups, Load Balancer, ECR)
- Auto Scaling Groups support added
- AWS SDK-only migration in progress through staged PRs

---

## Phase 1: Foundation (Completed)

> Core TUI and AWS integration

| Task | Description | Status |
|------|-------------|--------|
| TUI Framework | ratatui-based terminal UI | ✅ |
| AWS SDK Integration | EC2, VPC, SG, LB, ECR | ✅ |
| Blueprint System | Multi-region/service resource aggregation | ✅ |
| Markdown Generation | Auto-generate resource documentation | ✅ |
| Mermaid Diagrams | VPC network diagrams | ✅ |
| Multi-language | Korean/English support | ✅ |
| Self-Update | `emd update` command | ✅ |
| 12 Regions | Major AWS regions | ✅ |

---

## Phase 2-1: AWS SDK Consistency and Test-First Release Readiness (Priority)

| Task | Description | Status |
|------|-------------|--------|
| Large-Scale Testing | Reach 85% total line coverage by `cargo llvm-cov` + automate at least 17/20 core ASG/Launch Template/ECR scenarios | 🔄 |
| Test Scenario Catalog | Finalize 20 scenarios with priority labels (High/Medium) and test types (Unit/Integration/E2E) | 🔄 |
| Unit Tests 40% | Implement automated tests for parser/transform/sorting/error handling | 🔄 |
| Integration Tests 40% | Implement list -> detail -> blueprint flow automation | 🔄 |
| Regression/E2E Tests 20% | Implement region switch/refresh/detail-sync regression coverage | 🔄 |
| CI Gate Hardening | Block merge on missing critical scenarios + enforce `cargo llvm-cov --workspace --all-features --fail-under-lines 85` + keep lint gates | 🔄 |
| AWS SDK-only Refactor | Remove `aws` CLI command execution and standardize on SDK clients (EC2/VPC/SG/LB/ECR/IAM/ASG) | 🔄 |
| v0.1.1 Release Wrap-up | Complete release-blocking checks and create `v0.1.1` release tag after test gates pass | 🔄 |

### Phase 2-1 Test Scenario Catalog (20)

| Task | Description | Status |
|------|-------------|--------|
| ASG Scenarios (7) | list/detail/tags/scaling policies/instance states/error handling/sorting | 🔄 |
| Launch Template Scenarios (7) | list/detail/default-latest version/cross-region/block device/AMI-SG mapping/errors | 🔄 |
| ECR Scenarios (6) | repository list/image digest-tag sync/metadata gaps/duplicate tags/sorting/errors | 🔄 |

### Phase 2-1 Execution PR Split

| Task | Description | Status |
|------|-------------|--------|
| PR-1 Foundation | `feat(aws-sdk): foundation dispatcher and sts auth migration` | ✅ |
| PR-2 Core Services | `feat(aws-sdk): core services migration (ec2, vpc, security-group, ecr)` | 🔄 |
| PR-3 Remaining Services | `feat(aws-sdk): remaining services migration (elbv2, iam) and cleanup` | 🔄 |
| PR Quality Gate | Every PR must pass `cargo fmt --all` + `cargo clippy --all-targets --all-features -- -D warnings` + `cargo llvm-cov --workspace --all-features --fail-under-lines 85` + `./rust-lint-cleanup.sh` | 🔄 |

### Phase 2-1 Execution Order (Test-First)

| Task | Description | Status |
|------|-------------|--------|
| Scenario Finalization | Finalize ASG/Launch Template/ECR 20-scenario catalog with priority tags | 🔄 |
| Test Implementation | Deliver automated tests to satisfy Unit/Integration/Regression-E2E mix while reaching 85% workspace coverage | 🔄 |
| CI Gate Enforcement | Block merge when critical scenario coverage is missing or coverage is below 85% | 🔄 |
| Release Closure | Close `v0.1.1` tag/release checklist after test gates pass | 🔄 |

## Phase 2-2: AWS Service Expansion (In Progress)

| Task | Description | Status |
|------|-------------|--------|
| Auto Scaling Groups | ASG information and policies | ✅ |
| ASG Policy/Metadata | Validate Launch Template linkage, tags, and instance-state exposure | ✅ |
| Launch Templates | Launch template list and detail parity checks | 🔄 |
| Launch Template Template | Standardize detail fields (AMI, instance type, SG, block device, etc.) | 🔄 |
| Launch Template Error Handling | Define failure messages, empty-value handling, and retry behavior | 🔄 |
| ECR Image Details | Align repository image tags, digests, and metadata sync | 🔄 |

## Phase 2-3: AWS Service Expansion (Backlog)

| Task | Description | Status |
|------|-------------|--------|
| RDS | Database instances/clusters |  |
| RDS Detail Template | Draft key fields for RDS markdown details |  |
| Lambda | Functions list retrieval and error handling |  |
| Lambda Detail Items | Define environment variables, triggers, runtime, source mapping |  |
| S3 | Bucket list pagination and retrieval policy |  |
| S3 Detail Items | Define policy/encryption/ACL exposure scope |  |
| CloudFront | Model distribution state, domain, and origin structure |  |
| Route53 | Normalize hosted zone and record retrieval |  |
| ECS/EKS | Review common format abstraction for services/clusters |  |
| ElastiCache | Define visibility fields for cluster type, endpoints, and SG |  |
| DynamoDB | Define table/index/capacity parity fields |  |
| SNS/SQS | Decide queue/topic/subscription relationship notation |  |

---

## v0.2.0 Release

> Upon Phase 2 completion
>
> **Key Features**: 10+ major AWS services supported
>
> **Release Gate**: Must reach 85% total line coverage (`cargo llvm-cov`) before shipping

---

## Phase 3: Enhanced Documentation

> Multiple formats and advanced documentation

| Task | Description | Status |
|------|-------------|--------|
| PDF Export | Markdown to PDF conversion |  |
| HTML Export | Static HTML documentation |  |
| Custom Templates | User-defined document templates |  |
| Architecture Diagrams | Full infrastructure visualization (Mermaid) |  |
| Resource Tags | Tag-based classification and display |  |
| Cost Information | Estimated costs per resource (optional) |  |

---

## Phase 4: UX Improvements

> Enhanced user experience

| Task | Description | Status |
|------|-------------|--------|
| Search Functionality | Filter resources by name/tag |  |
| Sort Options | Sort by name, creation date, region |  |
| Favorites | Bookmark frequently used resources |  |
| Theme Support | Light/dark mode, custom colors |  |
| Custom Keybindings | User-defined keyboard shortcuts |  |
| Multi-profile | Quick AWS profile switching |  |
| History | Recently viewed resources |  |
| Batch Operations | Select and document multiple resources |  |

---

## v0.3.0 Release

> Upon Phase 3-4 completion
>
> **Key Features**: Multiple format support + Enhanced UX

---

## Phase 5: Advanced Features

> Analysis and collaboration

| Task | Description | Status |
|------|-------------|--------|
| Compare Mode | Compare resources across regions/accounts |  |
| Change Detection | Track resource modifications |  |
| Tag Management | Bulk tag viewing and suggestions |  |
| Cost Analysis | AWS Cost Explorer integration |  |
| Compliance Checks | Security group rule auditing |  |
| Resource Graph | Dependency visualization |  |
| Team Blueprints | Git-based blueprint sharing |  |
| Annotations | Add notes to resources |  |
| Snapshots | Save infrastructure state at specific points |  |
| Diff Comparison | Compare changes between snapshots |  |

---

## Phase 6: IaC Integration

> Infrastructure as Code integration

| Task | Description | Status |
|------|-------------|--------|
| Terraform Read | Import resources from TF files |  |
| Terraform Generate | Convert infrastructure to TF code (basic) |  |
| CloudFormation | Generate docs from CF templates |  |
| CDK Support | AWS CDK stack analysis |  |

---

## v0.4.0 Release

> Upon Phase 5-6 completion
>
> **Key Features**: Advanced analysis + IaC integration

---

## Phase 7: Platform Expansion

> Multi-cloud and enterprise

| Task | Description | Status |
|------|-------------|--------|
| GCP Support | Google Cloud Platform |  |
| Azure Support | Microsoft Azure |  |
| Unified View | Multi-cloud dashboard |  |
| CI/CD Integration | GitHub Actions/GitLab CI plugins |  |
| API Server | RESTful API endpoints |  |
| Web UI | Browser-based interface (optional) |  |

---

## Phase 8: Enterprise Features

> Large organization support

| Task | Description | Status |
|------|-------------|--------|
| RBAC | Role-based access control |  |
| Audit Logs | Track all operations |  |
| SSO Integration | SAML/OIDC authentication |  |
| Central Management | Unified multi-account management |  |

---

## v1.0.0 Release

> Upon Phase 7-8 completion
>
> **Key Features**: Multi-cloud + Enterprise features

---

## Next Steps

Current priorities for Phase 2 (must complete before v0.2.0):

1. Finalize the 20-scenario catalog for ASG/Launch Template/ECR with priority labels
2. Implement automated tests to cover at least 17 of the 20 core scenarios
3. Enforce CI merge gates for scenario coverage + lint quality
4. Complete `v0.1.1` release closure after all test gates are green

### Large-Scale Testing Details

Goal: **85% total line coverage** with priority-based scenarios

1. Scenario basis: 20 core scenarios total (ASG 7, Launch Template 7, ECR 6)
2. Coverage target: 17 of 20 scenarios automated and stable in CI
3. Required test mix: Unit tests 40%, integration tests 40%, regression/E2E 20%
4. Mandatory scenario set: ASG scale/state sync, Launch Template version/default/region lookup, ECR tag/metadata/detail consistency
5. PR gate: Merge blocked if added/modified feature lacks mapped core scenario coverage, or if `cargo llvm-cov` drops below 85%

Acceptance: Zero failures in target modules and full green result across all CI test stages.

---

## Milestones

| Version | Goal | Phase | Timeline |
|---------|------|-------|----------|
| v0.1.1 | Current version (core features) | Phase 1 | ✅ Done |
| v0.2.0 | AWS service expansion | Phase 2 | 2026 Q2 |
| v0.3.0 | Documentation + UX | Phase 3-4 | 2026 Q3 |
| v0.4.0 | Advanced features + IaC | Phase 5-6 | 2026 Q4 |
| v1.0.0 | Multi-cloud + Enterprise | Phase 7-8 | 2027 |

---

## Notes

- This roadmap is subject to change based on actual development
- Priorities may be adjusted based on user feedback and community needs
- See [CHANGELOG.md](CHANGELOG.md) for detailed version history

---

## Contributing

- Issues: https://github.com/SteelCrab/emd/issues
- Discussions: https://github.com/SteelCrab/emd/discussions
- Pull requests welcome

---

**Last Updated**: 2026-02-13
