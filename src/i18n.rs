use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum Language {
    Korean,
    #[default]
    English,
}

impl Language {
    pub fn display(&self) -> &'static str {
        match self {
            Language::Korean => "한국어",
            Language::English => "English",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Language::Korean => Language::English,
            Language::English => Language::Korean,
        }
    }
}

pub struct I18n {
    pub lang: Language,
}

impl I18n {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }

    // Common UI
    pub fn exit(&self) -> &'static str {
        match self.lang {
            Language::Korean => "종료",
            Language::English => "Exit",
        }
    }

    pub fn settings(&self) -> &'static str {
        match self.lang {
            Language::Korean => "설정",
            Language::English => "Settings",
        }
    }

    pub fn main_tab(&self) -> &'static str {
        match self.lang {
            Language::Korean => "메인",
            Language::English => "Main",
        }
    }

    pub fn back(&self) -> &'static str {
        match self.lang {
            Language::Korean => "뒤로",
            Language::English => "Back",
        }
    }

    pub fn select(&self) -> &'static str {
        match self.lang {
            Language::Korean => "선택",
            Language::English => "Select",
        }
    }

    pub fn move_cursor(&self) -> &'static str {
        match self.lang {
            Language::Korean => "이동",
            Language::English => "Move",
        }
    }

    pub fn refresh(&self) -> &'static str {
        match self.lang {
            Language::Korean => "새로고침",
            Language::English => "Refresh",
        }
    }

    pub fn save(&self) -> &'static str {
        match self.lang {
            Language::Korean => "저장",
            Language::English => "Save",
        }
    }

    pub fn delete(&self) -> &'static str {
        match self.lang {
            Language::Korean => "삭제",
            Language::English => "Delete",
        }
    }

    pub fn add(&self) -> &'static str {
        match self.lang {
            Language::Korean => "추가",
            Language::English => "Add",
        }
    }

    pub fn cancel(&self) -> &'static str {
        match self.lang {
            Language::Korean => "취소",
            Language::English => "Cancel",
        }
    }

    pub fn confirm(&self) -> &'static str {
        match self.lang {
            Language::Korean => "확인",
            Language::English => "Confirm",
        }
    }

    pub fn scroll(&self) -> &'static str {
        match self.lang {
            Language::Korean => "스크롤",
            Language::English => "Scroll",
        }
    }

    pub fn page(&self) -> &'static str {
        match self.lang {
            Language::Korean => "페이지",
            Language::English => "Page",
        }
    }

    pub fn generate(&self) -> &'static str {
        match self.lang {
            Language::Korean => "생성",
            Language::English => "Generate",
        }
    }

    pub fn reorder(&self) -> &'static str {
        match self.lang {
            Language::Korean => "순서변경",
            Language::English => "Reorder",
        }
    }

    pub fn retry(&self) -> &'static str {
        match self.lang {
            Language::Korean => "재시도",
            Language::English => "Retry",
        }
    }

    pub fn single_mode(&self) -> &'static str {
        match self.lang {
            Language::Korean => "단일 모드",
            Language::English => "Single Mode",
        }
    }

    pub fn add_to_blueprint(&self) -> &'static str {
        match self.lang {
            Language::Korean => "블루프린터에 추가",
            Language::English => "Add to Blueprint",
        }
    }

    pub fn markdown_generate(&self) -> &'static str {
        match self.lang {
            Language::Korean => "마크다운 생성",
            Language::English => "Generate Markdown",
        }
    }

    // Screen titles
    pub fn login(&self) -> &'static str {
        match self.lang {
            Language::Korean => "로그인",
            Language::English => "Login",
        }
    }

    pub fn region(&self) -> &'static str {
        match self.lang {
            Language::Korean => "리전",
            Language::English => "Region",
        }
    }

    pub fn service(&self) -> &'static str {
        match self.lang {
            Language::Korean => "서비스",
            Language::English => "Service",
        }
    }

    pub fn blueprint(&self) -> &'static str {
        match self.lang {
            Language::Korean => "블루프린터",
            Language::English => "Blueprint",
        }
    }

    pub fn preview(&self) -> &'static str {
        match self.lang {
            Language::Korean => "미리보기",
            Language::English => "Preview",
        }
    }

    pub fn loading(&self) -> &'static str {
        match self.lang {
            Language::Korean => "로딩",
            Language::English => "Loading",
        }
    }

    // Messages
    pub fn loading_msg(&self) -> &'static str {
        match self.lang {
            Language::Korean => "로딩 중...",
            Language::English => "Loading...",
        }
    }

    pub fn aws_cli_waiting(&self) -> &'static str {
        match self.lang {
            Language::Korean => "AWS CLI 응답 대기 중입니다.",
            Language::English => "Waiting for AWS CLI response.",
        }
    }

    pub fn refresh_complete(&self) -> &'static str {
        match self.lang {
            Language::Korean => "새로고침 완료",
            Language::English => "Refresh complete",
        }
    }

    pub fn save_complete(&self) -> &'static str {
        match self.lang {
            Language::Korean => "저장 완료",
            Language::English => "Save complete",
        }
    }

    pub fn resource_added(&self) -> &'static str {
        match self.lang {
            Language::Korean => "리소스 추가 완료",
            Language::English => "Resource added",
        }
    }

    pub fn resource_deleted(&self) -> &'static str {
        match self.lang {
            Language::Korean => "리소스 삭제 완료",
            Language::English => "Resource deleted",
        }
    }

    pub fn blueprint_saved(&self) -> &'static str {
        match self.lang {
            Language::Korean => "블루프린터 저장 완료",
            Language::English => "Blueprint saved",
        }
    }

    pub fn blueprint_deleted(&self) -> &'static str {
        match self.lang {
            Language::Korean => "블루프린터 삭제 완료",
            Language::English => "Blueprint deleted",
        }
    }

    pub fn no_resources(&self) -> &'static str {
        match self.lang {
            Language::Korean => "리소스가 없습니다",
            Language::English => "No resources",
        }
    }

    pub fn no_instances(&self) -> &'static str {
        match self.lang {
            Language::Korean => "인스턴스가 없습니다.",
            Language::English => "No instances found.",
        }
    }

    pub fn no_vpcs(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Network가 없습니다.",
            Language::English => "No networks found.",
        }
    }

    pub fn no_security_groups(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Security Group이 없습니다.",
            Language::English => "No security groups found.",
        }
    }

    pub fn no_load_balancers(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Load Balancer가 없습니다.",
            Language::English => "No load balancers found.",
        }
    }

    pub fn no_ecr_repos(&self) -> &'static str {
        match self.lang {
            Language::Korean => "ECR 레포지토리가 없습니다.",
            Language::English => "No ECR repositories found.",
        }
    }

    // Login messages
    pub fn aws_login_verified(&self) -> &'static str {
        match self.lang {
            Language::Korean => "✓ AWS 로그인 확인됨",
            Language::English => "✓ AWS login verified",
        }
    }

    pub fn aws_login_required(&self) -> &'static str {
        match self.lang {
            Language::Korean => "✗ AWS 로그인 필요",
            Language::English => "✗ AWS login required",
        }
    }

    pub fn aws_login_checking(&self) -> &'static str {
        match self.lang {
            Language::Korean => "AWS CLI 로그인 확인 중...",
            Language::English => "Checking AWS CLI login...",
        }
    }

    pub fn aws_configure_hint(&self) -> &'static str {
        match self.lang {
            Language::Korean => "aws configure 또는 aws sso login을 실행하세요.",
            Language::English => "Run 'aws configure' or 'aws sso login'.",
        }
    }

    // Loading tasks
    pub fn processing(&self) -> &'static str {
        match self.lang {
            Language::Korean => "처리 중",
            Language::English => "Processing",
        }
    }

    pub fn refreshing_ec2_list(&self) -> &'static str {
        match self.lang {
            Language::Korean => "EC2 목록 새로고침 중",
            Language::English => "Refreshing EC2 list",
        }
    }

    pub fn refreshing_vpc_list(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Network 목록 새로고침 중",
            Language::English => "Refreshing Network list",
        }
    }

    pub fn refreshing_sg_list(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Security Group 목록 새로고침 중",
            Language::English => "Refreshing Security Group list",
        }
    }

    pub fn refreshing_preview(&self) -> &'static str {
        match self.lang {
            Language::Korean => "미리보기 새로고침 중",
            Language::English => "Refreshing preview",
        }
    }

    pub fn loading_ec2_list(&self) -> &'static str {
        match self.lang {
            Language::Korean => "EC2 인스턴스 목록 조회 중",
            Language::English => "Loading EC2 instances",
        }
    }

    pub fn loading_vpc_list(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Network(VPC) 목록 조회 중",
            Language::English => "Loading Networks (VPC)",
        }
    }

    pub fn loading_sg_list(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Security Group 목록 조회 중",
            Language::English => "Loading Security Groups",
        }
    }

    pub fn loading_ec2_detail(&self) -> &'static str {
        match self.lang {
            Language::Korean => "EC2 상세 정보 조회 중",
            Language::English => "Loading EC2 details",
        }
    }

    pub fn loading_vpc_detail(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Network 상세 정보 조회 중",
            Language::English => "Loading Network details",
        }
    }

    pub fn loading_sg_detail(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Security Group 상세 정보 조회 중",
            Language::English => "Loading Security Group details",
        }
    }

    pub fn refreshing_lb_list(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Load Balancer 목록 새로고침 중",
            Language::English => "Refreshing Load Balancer list",
        }
    }

    pub fn loading_lb_list(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Load Balancer 목록 조회 중",
            Language::English => "Loading Load Balancers",
        }
    }

    pub fn loading_lb_detail(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Load Balancer 상세 정보 조회 중",
            Language::English => "Loading Load Balancer details",
        }
    }

    pub fn refreshing_ecr_list(&self) -> &'static str {
        match self.lang {
            Language::Korean => "ECR 목록 새로고침 중",
            Language::English => "Refreshing ECR list",
        }
    }

    pub fn loading_ecr_list(&self) -> &'static str {
        match self.lang {
            Language::Korean => "ECR 레포지토리 목록 조회 중",
            Language::English => "Loading ECR repositories",
        }
    }

    pub fn loading_ecr_detail(&self) -> &'static str {
        match self.lang {
            Language::Korean => "ECR 상세 정보 조회 중",
            Language::English => "Loading ECR details",
        }
    }

    pub fn loading_blueprint_resources(&self) -> &'static str {
        match self.lang {
            Language::Korean => "블루프린트 리소스 조회 중",
            Language::English => "Loading Blueprint resources",
        }
    }

    // VPC Loading steps
    pub fn vpc_basic_info(&self) -> &'static str {
        match self.lang {
            Language::Korean => "VPC 기본 정보",
            Language::English => "VPC Basic Info",
        }
    }

    pub fn subnets(&self) -> &'static str {
        match self.lang {
            Language::Korean => "서브넷",
            Language::English => "Subnets",
        }
    }

    pub fn internet_gateway(&self) -> &'static str {
        match self.lang {
            Language::Korean => "인터넷 게이트웨이",
            Language::English => "Internet Gateway",
        }
    }

    pub fn nat_gateway(&self) -> &'static str {
        match self.lang {
            Language::Korean => "NAT 게이트웨이",
            Language::English => "NAT Gateway",
        }
    }

    pub fn route_tables(&self) -> &'static str {
        match self.lang {
            Language::Korean => "라우팅 테이블",
            Language::English => "Route Tables",
        }
    }

    pub fn elastic_ip(&self) -> &'static str {
        match self.lang {
            Language::Korean => "Elastic IP",
            Language::English => "Elastic IP",
        }
    }

    pub fn dns_settings(&self) -> &'static str {
        match self.lang {
            Language::Korean => "DNS 설정",
            Language::English => "DNS Settings",
        }
    }

    pub fn completing(&self) -> &'static str {
        match self.lang {
            Language::Korean => "완료 중",
            Language::English => "Completing",
        }
    }

    pub fn current_loading(&self, task: &str) -> String {
        match self.lang {
            Language::Korean => format!("현재: {} 조회 중...", task),
            Language::English => format!("Current: Loading {}...", task),
        }
    }

    // Blueprint
    pub fn new_blueprint(&self) -> &'static str {
        match self.lang {
            Language::Korean => "+ 새 블루프린터",
            Language::English => "+ New Blueprint",
        }
    }

    pub fn blueprint_load_failed(&self) -> &'static str {
        match self.lang {
            Language::Korean => "블루프린터 로드 실패",
            Language::English => "Blueprint load failed",
        }
    }

    pub fn enter_blueprint_name(&self) -> &'static str {
        match self.lang {
            Language::Korean => "블루프린터 이름을 입력하세요:",
            Language::English => "Enter blueprint name:",
        }
    }

    pub fn press_a_to_add(&self) -> &'static str {
        match self.lang {
            Language::Korean => "'a' 키를 눌러 리소스를 추가하세요.",
            Language::English => "Press 'a' to add resources.",
        }
    }

    pub fn resources(&self) -> &'static str {
        match self.lang {
            Language::Korean => "리소스",
            Language::English => "resources",
        }
    }

    // Settings
    pub fn language(&self) -> &'static str {
        match self.lang {
            Language::Korean => "언어",
            Language::English => "Language",
        }
    }

    #[allow(dead_code)]
    pub fn language_setting(&self) -> &'static str {
        match self.lang {
            Language::Korean => "언어 설정",
            Language::English => "Language Setting",
        }
    }

    pub fn settings_saved(&self) -> &'static str {
        match self.lang {
            Language::Korean => "설정 저장 완료",
            Language::English => "Settings saved",
        }
    }

    pub fn change(&self) -> &'static str {
        match self.lang {
            Language::Korean => "변경",
            Language::English => "Change",
        }
    }

    // Table headers for markdown
    pub fn item(&self) -> &'static str {
        match self.lang {
            Language::Korean => "항목",
            Language::English => "Item",
        }
    }

    pub fn value(&self) -> &'static str {
        match self.lang {
            Language::Korean => "값",
            Language::English => "Value",
        }
    }

    pub fn md_name(&self) -> &'static str {
        match self.lang {
            Language::Korean => "이름",
            Language::English => "Name",
        }
    }

    pub fn md_state(&self) -> &'static str {
        match self.lang {
            Language::Korean => "상태",
            Language::English => "State",
        }
    }

    pub fn tag(&self) -> &'static str {
        match self.lang {
            Language::Korean => "태그",
            Language::English => "Tag",
        }
    }

    // Toc
    pub fn toc(&self) -> &'static str {
        match self.lang {
            Language::Korean => "📑 목차",
            Language::English => "📑 Table of Contents",
        }
    }

    // Query failed
    pub fn query_failed(&self) -> &'static str {
        match self.lang {
            Language::Korean => "조회 실패",
            Language::English => "Query failed",
        }
    }

    // VPC/Network markdown labels
    pub fn md_dns_support(&self) -> &'static str {
        match self.lang {
            Language::Korean => "DNS 지원",
            Language::English => "DNS Support",
        }
    }

    pub fn md_dns_hostnames(&self) -> &'static str {
        match self.lang {
            Language::Korean => "DNS 호스트 이름",
            Language::English => "DNS Hostnames",
        }
    }

    pub fn md_subnets(&self) -> &'static str {
        match self.lang {
            Language::Korean => "서브넷",
            Language::English => "Subnets",
        }
    }

    pub fn md_internet_gateway(&self) -> &'static str {
        match self.lang {
            Language::Korean => "인터넷 게이트웨이",
            Language::English => "Internet Gateway",
        }
    }

    pub fn md_attached_vpc(&self) -> &'static str {
        match self.lang {
            Language::Korean => "연결된 VPC",
            Language::English => "Attached VPC",
        }
    }

    pub fn md_nat_gateway(&self) -> &'static str {
        match self.lang {
            Language::Korean => "NAT 게이트웨이",
            Language::English => "NAT Gateway",
        }
    }

    pub fn md_availability_mode(&self) -> &'static str {
        match self.lang {
            Language::Korean => "가용성 모드",
            Language::English => "Availability Mode",
        }
    }

    pub fn md_zonal(&self) -> &'static str {
        match self.lang {
            Language::Korean => "영역",
            Language::English => "Zonal",
        }
    }

    pub fn md_regional(&self) -> &'static str {
        match self.lang {
            Language::Korean => "리전별",
            Language::English => "Regional",
        }
    }

    pub fn md_ip_auto_scaling(&self) -> &'static str {
        match self.lang {
            Language::Korean => "IP 자동 확장",
            Language::English => "IP Auto Scaling",
        }
    }

    pub fn md_zone_auto_provisioning(&self) -> &'static str {
        match self.lang {
            Language::Korean => "영역 자동 프로비저닝",
            Language::English => "Zone Auto Provisioning",
        }
    }

    pub fn md_enabled(&self) -> &'static str {
        match self.lang {
            Language::Korean => "활성화",
            Language::English => "Enabled",
        }
    }

    pub fn md_disabled(&self) -> &'static str {
        match self.lang {
            Language::Korean => "비활성화",
            Language::English => "Disabled",
        }
    }

    pub fn md_subnet(&self) -> &'static str {
        match self.lang {
            Language::Korean => "서브넷",
            Language::English => "Subnet",
        }
    }

    pub fn md_connectivity_type(&self) -> &'static str {
        match self.lang {
            Language::Korean => "연결 유형",
            Language::English => "Connectivity Type",
        }
    }

    pub fn md_public(&self) -> &'static str {
        match self.lang {
            Language::Korean => "퍼블릭",
            Language::English => "Public",
        }
    }

    pub fn md_private(&self) -> &'static str {
        match self.lang {
            Language::Korean => "프라이빗",
            Language::English => "Private",
        }
    }

    pub fn md_elastic_ip_allocation_id(&self) -> &'static str {
        match self.lang {
            Language::Korean => "탄력적 IP 할당 ID",
            Language::English => "Elastic IP Allocation ID",
        }
    }

    pub fn md_route_tables(&self) -> &'static str {
        match self.lang {
            Language::Korean => "라우팅 테이블",
            Language::English => "Route Tables",
        }
    }

    pub fn md_destination(&self) -> &'static str {
        match self.lang {
            Language::Korean => "대상",
            Language::English => "Destination",
        }
    }

    pub fn md_target(&self) -> &'static str {
        match self.lang {
            Language::Korean => "대상",
            Language::English => "Target",
        }
    }

    pub fn md_associated_subnets(&self) -> &'static str {
        match self.lang {
            Language::Korean => "연결된 서브넷:",
            Language::English => "Associated Subnets:",
        }
    }

    pub fn md_association(&self) -> &'static str {
        match self.lang {
            Language::Korean => "연결",
            Language::English => "Association",
        }
    }

    pub fn md_network_diagram(&self) -> &'static str {
        match self.lang {
            Language::Korean => "네트워크 구성도",
            Language::English => "Network Diagram",
        }
    }

    // Security Group markdown labels
    pub fn md_description(&self) -> &'static str {
        match self.lang {
            Language::Korean => "설명",
            Language::English => "Description",
        }
    }

    pub fn md_inbound_rules(&self) -> &'static str {
        match self.lang {
            Language::Korean => "인바운드 규칙",
            Language::English => "Inbound Rules",
        }
    }

    pub fn md_outbound_rules(&self) -> &'static str {
        match self.lang {
            Language::Korean => "아웃바운드 규칙",
            Language::English => "Outbound Rules",
        }
    }

    pub fn md_protocol(&self) -> &'static str {
        match self.lang {
            Language::Korean => "프로토콜",
            Language::English => "Protocol",
        }
    }

    pub fn md_port_range(&self) -> &'static str {
        match self.lang {
            Language::Korean => "포트 범위",
            Language::English => "Port Range",
        }
    }

    pub fn md_source(&self) -> &'static str {
        match self.lang {
            Language::Korean => "소스",
            Language::English => "Source",
        }
    }

    // Load Balancer markdown labels
    pub fn md_dns_name(&self) -> &'static str {
        match self.lang {
            Language::Korean => "DNS 이름",
            Language::English => "DNS Name",
        }
    }

    pub fn md_type(&self) -> &'static str {
        match self.lang {
            Language::Korean => "타입",
            Language::English => "Type",
        }
    }

    pub fn md_ip_address_type(&self) -> &'static str {
        match self.lang {
            Language::Korean => "IP 주소 유형",
            Language::English => "IP Address Type",
        }
    }

    pub fn md_port(&self) -> &'static str {
        match self.lang {
            Language::Korean => "포트",
            Language::English => "Port",
        }
    }

    pub fn md_default_action(&self) -> &'static str {
        match self.lang {
            Language::Korean => "기본 액션",
            Language::English => "Default Action",
        }
    }

    pub fn md_basic_info(&self) -> &'static str {
        match self.lang {
            Language::Korean => "기본 정보:",
            Language::English => "Basic Info:",
        }
    }

    // EC2 markdown labels
    pub fn md_ec2_instance(&self) -> &'static str {
        match self.lang {
            Language::Korean => "EC2 인스턴스",
            Language::English => "EC2 Instance",
        }
    }

    pub fn md_instance_type(&self) -> &'static str {
        match self.lang {
            Language::Korean => "인스턴스 유형",
            Language::English => "Instance Type",
        }
    }

    pub fn md_platform(&self) -> &'static str {
        match self.lang {
            Language::Korean => "플랫폼",
            Language::English => "Platform",
        }
    }

    pub fn md_architecture(&self) -> &'static str {
        match self.lang {
            Language::Korean => "아키텍처",
            Language::English => "Architecture",
        }
    }

    pub fn md_key_pair(&self) -> &'static str {
        match self.lang {
            Language::Korean => "키 페어",
            Language::English => "Key Pair",
        }
    }

    pub fn md_availability_zone(&self) -> &'static str {
        match self.lang {
            Language::Korean => "가용 영역",
            Language::English => "Availability Zone",
        }
    }

    pub fn md_availability_zones(&self) -> &'static str {
        match self.lang {
            Language::Korean => "가용 영역",
            Language::English => "Availability Zones",
        }
    }

    pub fn md_private_ip(&self) -> &'static str {
        match self.lang {
            Language::Korean => "프라이빗 IP",
            Language::English => "Private IP",
        }
    }

    pub fn md_public_ip(&self) -> &'static str {
        match self.lang {
            Language::Korean => "퍼블릭 IP",
            Language::English => "Public IP",
        }
    }

    pub fn md_security_groups(&self) -> &'static str {
        match self.lang {
            Language::Korean => "보안 그룹",
            Language::English => "Security Groups",
        }
    }

    pub fn md_ebs_optimized(&self) -> &'static str {
        match self.lang {
            Language::Korean => "EBS 최적화",
            Language::English => "EBS Optimized",
        }
    }

    pub fn md_monitoring(&self) -> &'static str {
        match self.lang {
            Language::Korean => "모니터링",
            Language::English => "Monitoring",
        }
    }

    pub fn md_iam_role(&self) -> &'static str {
        match self.lang {
            Language::Korean => "IAM 역할",
            Language::English => "IAM Role",
        }
    }

    pub fn md_launch_time(&self) -> &'static str {
        match self.lang {
            Language::Korean => "시작 시간",
            Language::English => "Launch Time",
        }
    }

    pub fn md_storage(&self) -> &'static str {
        match self.lang {
            Language::Korean => "스토리지",
            Language::English => "Storage",
        }
    }

    pub fn md_device(&self) -> &'static str {
        match self.lang {
            Language::Korean => "디바이스",
            Language::English => "Device",
        }
    }

    pub fn md_size(&self) -> &'static str {
        match self.lang {
            Language::Korean => "크기",
            Language::English => "Size",
        }
    }

    pub fn md_encrypted(&self) -> &'static str {
        match self.lang {
            Language::Korean => "암호화",
            Language::English => "Encrypted",
        }
    }

    pub fn md_delete_on_termination(&self) -> &'static str {
        match self.lang {
            Language::Korean => "종료 시 삭제",
            Language::English => "Delete on Termination",
        }
    }

    pub fn md_user_data(&self) -> &'static str {
        match self.lang {
            Language::Korean => "사용자 데이터",
            Language::English => "User Data",
        }
    }

    // ECR markdown labels
    pub fn md_ecr_repository(&self) -> &'static str {
        match self.lang {
            Language::Korean => "ECR 레포지토리",
            Language::English => "ECR Repository",
        }
    }

    pub fn md_tag_mutability(&self) -> &'static str {
        match self.lang {
            Language::Korean => "태그 변경 가능",
            Language::English => "Tag Mutability",
        }
    }

    pub fn md_encryption(&self) -> &'static str {
        match self.lang {
            Language::Korean => "암호화",
            Language::English => "Encryption",
        }
    }

    pub fn md_image_count(&self) -> &'static str {
        match self.lang {
            Language::Korean => "이미지 수",
            Language::English => "Image Count",
        }
    }

    pub fn md_created_at(&self) -> &'static str {
        match self.lang {
            Language::Korean => "생성일",
            Language::English => "Created At",
        }
    }

    // Load Balancer extra labels
    pub fn md_scheme(&self) -> &'static str {
        match self.lang {
            Language::Korean => "스키마",
            Language::English => "Scheme",
        }
    }

    pub fn md_target_type(&self) -> &'static str {
        match self.lang {
            Language::Korean => "대상 유형",
            Language::English => "Target Type",
        }
    }

    pub fn md_health_check(&self) -> &'static str {
        match self.lang {
            Language::Korean => "헬스 체크",
            Language::English => "Health Check",
        }
    }

    pub fn md_threshold(&self) -> &'static str {
        match self.lang {
            Language::Korean => "임계값",
            Language::English => "Threshold",
        }
    }

    pub fn md_healthy(&self) -> &'static str {
        match self.lang {
            Language::Korean => "정상",
            Language::English => "Healthy",
        }
    }

    pub fn md_unhealthy(&self) -> &'static str {
        match self.lang {
            Language::Korean => "비정상",
            Language::English => "Unhealthy",
        }
    }

    pub fn md_targets(&self) -> &'static str {
        match self.lang {
            Language::Korean => "대상:",
            Language::English => "Targets:",
        }
    }
}
