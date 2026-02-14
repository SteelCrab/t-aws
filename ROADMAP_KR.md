# EMD Roadmap

> AWS 리소스 탐색 및 문서화 도구 개발 로드맵

## 현재 상태

- **기준 브랜치**: `main`
- **작업 브랜치**: `feat/large-scale-tests-coverage-85`, `feat/aws-sdk-foundation`, `feat/aws-sdk-core-services`, `feat/aws-sdk-remaining-services`
- **최신 버전**: v0.2.0
- 기본 AWS 서비스 지원 완료 (EC2, VPC, Security Groups, Load Balancer, ECR)
- Auto Scaling Groups 기능 추가
- AWS SDK-only 전환 브랜치 작업 진행 중

## Phase 1: 기반 구축 (완료)

| 작업 | 설명 | 상태 |
|------|------|------|
| TUI 프레임워크 | ratatui 기반 터미널 UI | ✅ |
| AWS SDK 통합 | EC2, VPC, SG, LB, ECR | ✅ |
| 블루프린트 시스템 | 멀티 리전/서비스 리소스 통합 | ✅ |
| 마크다운 생성 | 리소스 문서 자동 생성 | ✅ |
| Mermaid 다이어그램 | VPC 네트워크 구성도 | ✅ |
| 다국어 지원 | 한국어/영어 | ✅ |
| 자동 업데이트 | `emd update` 명령어 | ✅ |
| 12개 리전 | 주요 AWS 리전 지원 | ✅ |

## Phase 2-1: AWS SDK 정합성 및 테스트 우선 배포 준비 (우선순위)

| 작업 | 설명 | 상태 |
|------|------|------|
| 대규모 테스트 수준 | `cargo llvm-cov` 기준 전체 라인 커버리지 85% 달성 + ASG/Launch Template/ECR 핵심 시나리오 20개 중 17개 자동화 | 🔄 |
| 테스트 시나리오 카탈로그 | 20개 시나리오를 우선순위(High/Medium) + 테스트 타입(Unit/Integration/E2E)으로 확정 | 🔄 |
| 단위 테스트 40% | 파서/변환/정렬/오류 처리 중심 자동 테스트 구현 | 🔄 |
| 통합 테스트 40% | 목록 조회 → 상세 조회 → 블루프린트 반영 플로우 자동 테스트 구현 | 🔄 |
| 회귀/E2E 테스트 20% | 리전 전환/새로고침/상세 동기화 회귀 테스트 구현 | 🔄 |
| CI 게이트 강화 | 핵심 시나리오 미포함 PR 병합 차단 + `cargo llvm-cov --workspace --all-features --fail-under-lines 85` + lint 게이트 유지 | 🔄 |
| AWS SDK 전환 | 모든 AWS 조회 경로에서 `aws` CLI 호출 제거 후 SDK(EC2/VPC/SG/LB/ECR/IAM/ASG)만 사용 | 🔄 |
| v0.2.0 배포 마무리 | 테스트 기준 충족 후 `v0.2.0` 릴리스 태그 생성 및 배포 체크리스트 마감 | 🔄 |

### Phase 2-1 테스트 시나리오 카탈로그 (20개)

| 작업 | 설명 | 상태 |
|------|------|------|
| ASG 시나리오 (7) | 목록/상세/태그/스케일 정책/인스턴스 상태/오류 처리/정렬 | 🔄 |
| Launch Template 시나리오 (7) | 목록/상세/기본·최신 버전/교차 리전/Block Device/AMI·SG 매핑/오류 처리 | 🔄 |
| ECR 시나리오 (6) | 리포지토리 목록/이미지 상세 digest-tag/메타데이터 누락/중복 태그/정렬/오류 처리 | 🔄 |

### Phase 2-1 실행 PR 분할

| 작업 | 설명 | 상태 |
|------|------|------|
| PR-1 Foundation | `feat(aws-sdk): foundation dispatcher and sts auth migration` | ✅ |
| PR-2 Core Services | `feat(aws-sdk): core services migration (ec2, vpc, security-group, ecr)` | 🔄 |
| PR-3 Remaining Services | `feat(aws-sdk): remaining services migration (elbv2, iam) and cleanup` | 🔄 |
| PR 공통 게이트 | 각 PR마다 `cargo fmt --all` + `cargo clippy --all-targets --all-features -- -D warnings` + `cargo llvm-cov --workspace --all-features --fail-under-lines 85` + `./rust-lint-cleanup.sh` 통과 | 🔄 |

### Phase 2-1 실행 순서 (테스트 우선)

| 작업 | 설명 | 상태 |
|------|------|------|
| 시나리오 확정 | ASG/Launch Template/ECR 20개 시나리오 확정 및 우선순위 태깅 | 🔄 |
| 테스트 코드 구현 | Unit/Integration/Regression-E2E 비율을 유지하면서 `cargo llvm-cov` 전체 85% 달성 수준으로 자동 테스트 구현 | 🔄 |
| CI 게이트 적용 | 핵심 시나리오 미커버리지 병합 차단 + `cargo llvm-cov` 85% 미만 시 병합 차단 | 🔄 |
| 배포 마감 | 테스트 기준 충족 확인 후 `v0.2.0` 태그/릴리스 마감 | 🔄 |

## Phase 2-2: AWS 서비스 확장 (현재 정합화 대상)

| 작업 | 설명 | 상태 |
|------|------|------|
| Auto Scaling Groups | ASG 목록·상세 정합성 검증 | ✅ |
| ASG 정책/메타데이터 | Launch Template 연동 정책, 태그, 인스턴스 상태 노출 확인 | ✅ |
| Launch Template | 런치 템플릿 목록·상세 정합성 점검 | 🔄 |
| Launch Template 템플릿 | 상세 출력 항목(AMI, 인스턴스 타입, SG, Block Device 등) 표준화 | 🔄 |
| Launch Template 오류 처리 | 실패 케이스 메시지/빈값 처리/재시도 동작 정리 | 🔄 |
| ECR 이미지 상세 | 태그/메타데이터/메시지 동기화 범위 정리 | 🔄 |

## Phase 2-3: AWS 서비스 확장 (향후)

| 작업 | 설명 | 상태 |
|------|------|------|
| RDS | 데이터베이스 인스턴스/클러스터 조사 및 API 표준화 |  |
| RDS 상세 템플릿 | RDS 상세 마크다운 템플릿(핵심 속성) 초안 |  |
| Lambda | 함수 목록 조회 API/에러 처리 검토 |  |
| Lambda 상세 항목 | 환경변수·트리거·런타임·코드 저장소 링크 정의 |  |
| S3 | 버킷 목록 조회 및 페이지네이션 정책 |  |
| S3 상세 항목 | 정책/암호화/ACL 노출 범위 정의 |  |
| CloudFront | 배포 상태/도메인/오리진 구조 모델링 |  |
| Route53 | 호스팅 영역/레코드 조회 정규화 |  |
| ECS/EKS | 서비스/클러스터 공통 추상화 포맷 검토 |  |
| ElastiCache | 클러스터 타입/엔드포인트/보안 그룹 노출 항목 |  |
| DynamoDB | 테이블/인덱스/용량 항목 정합성 정의 |  |
| SNS/SQS | 큐/토픽/구독 관계표 표기 방식 결정 |  |

## Phase 3: 문서화 기능 강화

| 작업 | 설명 | 상태 |
|------|------|------|
| PDF 내보내기 | 마크다운 → PDF 변환 |  |
| HTML 내보내기 | 정적 HTML 문서 생성 |  |
| 커스텀 템플릿 | 사용자 정의 문서 템플릿 |  |
| 아키텍처 다이어그램 | 전체 인프라 시각화 (Mermaid) |  |
| 리소스 태그 정보 | 태그 기반 분류 및 표시 |  |
| 비용 정보 | 리소스별 예상 비용 (선택적) |  |

## Phase 4: UX 개선

| 작업 | 설명 | 상태 |
|------|------|------|
| 검색 기능 | 리소스 이름/태그 필터링 |  |
| 정렬 옵션 | 이름, 생성일, 리전별 정렬 |  |
| 즐겨찾기 | 자주 사용하는 리소스 북마크 |  |
| 테마 지원 | 라이트/다크 모드, 커스텀 컬러 |  |
| 단축키 커스터마이징 | 사용자 정의 키 바인딩 |  |
| 멀티 프로파일 | AWS 프로파일 간 빠른 전환 |  |
| 히스토리 | 최근 열어본 리소스 |  |
| 배치 작업 | 여러 리소스 동시 선택/문서화 |  |

## Phase 5: 고급 기능

| 작업 | 설명 | 상태 |
|------|------|------|
| 비교 모드 | 리전/계정 간 리소스 비교 |  |
| 변경 감지 | 리소스 변경 사항 추적 |  |
| 태깅 관리 | 태그 일괄 조회 및 제안 |  |
| 비용 분석 | AWS Cost Explorer 통합 |  |
| 컴플라이언스 체크 | 보안 그룹 규칙 감사 |  |
| 리소스 그래프 | 의존성 시각화 |  |
| 팀 블루프린트 | Git 기반 블루프린트 공유 |  |
| 주석/코멘트 | 리소스에 메모 추가 |  |
| 스냅샷 | 특정 시점 인프라 상태 저장 |  |
| 차이 비교 | 스냅샷 간 변경 내역 비교 |  |

## Phase 6: IaC 통합

| 작업 | 설명 | 상태 |
|------|------|------|
| Terraform 읽기 | TF 파일에서 리소스 가져오기 |  |
| Terraform 생성 | 현재 인프라 → TF 코드 변환 (기본) |  |
| CloudFormation | CF 템플릿 기반 문서 생성 |  |
| CDK 지원 | AWS CDK 스택 분석 |  |

## Phase 7: 플랫폼 확장

| 작업 | 설명 | 상태 |
|------|------|------|
| GCP 지원 | Google Cloud Platform |  |
| Azure 지원 | Microsoft Azure |  |
| 통합 뷰 | 멀티 클라우드 대시보드 |  |
| CI/CD 통합 | GitHub Actions/GitLab CI 플러그인 |  |
| API 서버 | RESTful API 제공 |  |
| 웹 UI | 브라우저 기반 인터페이스 (선택적) |  |

## Phase 8: 엔터프라이즈 기능

| 작업 | 설명 | 상태 |
|------|------|------|
| RBAC | 역할 기반 접근 제어 |  |
| 감사 로그 | 모든 작업 기록 |  |
| SSO 통합 | SAML/OIDC 인증 |  |
| 중앙 관리 | 여러 계정 통합 관리 |  |

**마지막 업데이트**: 2026-02-13
