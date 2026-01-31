use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Korean,
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

    // Table headers for markdown (reserved for future use)
    #[allow(dead_code)]
    pub fn item(&self) -> &'static str {
        match self.lang {
            Language::Korean => "항목",
            Language::English => "Item",
        }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> &'static str {
        match self.lang {
            Language::Korean => "값",
            Language::English => "Value",
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self.lang {
            Language::Korean => "이름",
            Language::English => "Name",
        }
    }

    #[allow(dead_code)]
    pub fn state(&self) -> &'static str {
        match self.lang {
            Language::Korean => "상태",
            Language::English => "State",
        }
    }

    #[allow(dead_code)]
    pub fn tag(&self) -> &'static str {
        match self.lang {
            Language::Korean => "태그",
            Language::English => "Tag",
        }
    }

    // Toc
    #[allow(dead_code)]
    pub fn toc(&self) -> &'static str {
        match self.lang {
            Language::Korean => "📑 목차",
            Language::English => "📑 Table of Contents",
        }
    }

    // Query failed
    #[allow(dead_code)]
    pub fn query_failed(&self) -> &'static str {
        match self.lang {
            Language::Korean => "조회 실패",
            Language::English => "Query failed",
        }
    }
}
