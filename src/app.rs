use crate::aws_cli::{
    self, AsgDetail, AwsAuthError, AwsAuthErrorCode, AwsResource, Ec2Detail, EcrDetail,
};
use crate::blueprint::{
    Blueprint, BlueprintResource, BlueprintStore, ResourceType, load_blueprints, save_blueprints,
};
use crate::i18n::{I18n, Language};
use crate::settings::{AppSettings, load_settings, save_settings};
use std::time::{Duration, Instant};

const LOGIN_SESSION_CHECK_INTERVAL: Duration = Duration::from_secs(15);

fn is_login_required_error(error: &AwsAuthError) -> bool {
    matches!(
        error.code,
        AwsAuthErrorCode::CredentialsProviderMissing
            | AwsAuthErrorCode::CredentialsLoadFailed
            | AwsAuthErrorCode::CallerIdentityFailed
    )
}

fn is_login_required_message(message: &str, i18n: &I18n) -> bool {
    message == i18n.auth_provider_missing()
        || message == i18n.auth_credentials_load_failed()
        || message == i18n.auth_caller_identity_failed()
}

fn is_aws_credential_message(message: &str, i18n: &I18n) -> bool {
    message == i18n.auth_network_error() || message == i18n.auth_unknown_error()
}

fn auth_error_message(error: &AwsAuthError, i18n: &I18n) -> String {
    match error.code {
        AwsAuthErrorCode::CredentialsProviderMissing => i18n.auth_provider_missing().to_string(),
        AwsAuthErrorCode::CredentialsLoadFailed => i18n.auth_credentials_load_failed().to_string(),
        AwsAuthErrorCode::CallerIdentityFailed => i18n.auth_caller_identity_failed().to_string(),
        AwsAuthErrorCode::Network => i18n.auth_network_error().to_string(),
        AwsAuthErrorCode::Unknown => i18n.auth_unknown_error().to_string(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Login,
    BlueprintSelect,
    BlueprintDetail,
    BlueprintNameInput,
    BlueprintPreview,
    RegionSelect,
    ServiceSelect,
    Ec2Select,
    VpcSelect,
    SecurityGroupSelect,
    LoadBalancerSelect,
    EcrSelect,
    AsgSelect,
    Preview,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadingTask {
    None,
    RefreshEc2,
    RefreshVpc,
    RefreshPreview,
    RefreshSecurityGroup,
    RefreshLoadBalancer,
    RefreshEcr,
    RefreshAsg,
    LoadEc2,
    LoadVpc,
    LoadSecurityGroup,
    LoadLoadBalancer,
    LoadEcr,
    LoadAsg,
    LoadEc2Detail(String),
    LoadVpcDetail(String, u8), // (vpc_id, step: 0-6)
    LoadSecurityGroupDetail(String),
    LoadLoadBalancerDetail(String),
    LoadEcrDetail(String),
    LoadAsgDetail(String),

    LoadBlueprintResources(usize), // (current_resource_index)
}

#[derive(Debug, Clone, Default)]
pub struct LoadingProgress {
    pub vpc_info: bool,
    pub subnets: bool,
    pub igws: bool,
    pub nats: bool,
    pub route_tables: bool,
    pub eips: bool,
    pub dns_attrs: bool,
}

impl LoadingProgress {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

pub struct Region {
    pub code: &'static str,
    pub name_ko: &'static str,
    pub name_en: &'static str,
}

impl Region {
    pub fn name(&self, lang: Language) -> &'static str {
        match lang {
            Language::Korean => self.name_ko,
            Language::English => self.name_en,
        }
    }
}

pub const REGIONS: &[Region] = &[
    Region {
        code: "ap-northeast-2",
        name_ko: "서울",
        name_en: "Seoul",
    },
    Region {
        code: "ap-northeast-1",
        name_ko: "도쿄",
        name_en: "Tokyo",
    },
    Region {
        code: "ap-northeast-3",
        name_ko: "오사카",
        name_en: "Osaka",
    },
    Region {
        code: "ap-southeast-1",
        name_ko: "싱가포르",
        name_en: "Singapore",
    },
    Region {
        code: "ap-southeast-2",
        name_ko: "시드니",
        name_en: "Sydney",
    },
    Region {
        code: "ap-south-1",
        name_ko: "뭄바이",
        name_en: "Mumbai",
    },
    Region {
        code: "us-east-1",
        name_ko: "버지니아",
        name_en: "N. Virginia",
    },
    Region {
        code: "us-east-2",
        name_ko: "오하이오",
        name_en: "Ohio",
    },
    Region {
        code: "us-west-1",
        name_ko: "캘리포니아",
        name_en: "N. California",
    },
    Region {
        code: "us-west-2",
        name_ko: "오레곤",
        name_en: "Oregon",
    },
    Region {
        code: "eu-west-1",
        name_ko: "아일랜드",
        name_en: "Ireland",
    },
    Region {
        code: "eu-central-1",
        name_ko: "프랑크푸르트",
        name_en: "Frankfurt",
    },
];

// Service names (excluding exit which is handled separately)
pub const SERVICE_KEYS: &[&str] = &[
    "EC2",
    "Network",
    "Security Group",
    "Load Balancer",
    "ECR",
    "ASG",
];

pub struct App {
    pub screen: Screen,
    pub running: bool,
    pub loading: bool,
    pub loading_task: LoadingTask,
    pub loading_progress: LoadingProgress,
    pub last_login_check: Option<Instant>,
    pub login_info: Option<String>,
    pub login_error: Option<String>,
    pub available_profiles: Vec<String>,
    pub selected_profile_index: usize,
    pub selected_region: usize,
    pub selected_service: usize,
    pub selected_index: usize,
    pub message: String,

    // Settings & i18n
    pub settings: AppSettings,
    pub i18n: I18n,
    pub selected_setting: usize,
    pub selected_tab: usize, // 0: Main, 1: Settings
    // AWS Resources
    pub instances: Vec<AwsResource>,
    pub vpcs: Vec<AwsResource>,
    pub security_groups: Vec<AwsResource>,
    pub load_balancers: Vec<AwsResource>,
    pub ecr_repositories: Vec<AwsResource>,
    pub auto_scaling_groups: Vec<AwsResource>,

    // Selected EC2 Detail
    pub ec2_detail: Option<Ec2Detail>,
    // Selected Network Detail
    pub network_detail: Option<aws_cli::NetworkDetail>,
    // Selected Security Group Detail
    pub sg_detail: Option<aws_cli::SecurityGroupDetail>,
    // Selected Load Balancer Detail
    pub lb_detail: Option<aws_cli::LoadBalancerDetail>,
    // Selected ECR Detail
    pub ecr_detail: Option<EcrDetail>,
    // Selected ASG Detail
    pub asg_detail: Option<AsgDetail>,

    // Preview
    pub preview_content: String,
    pub preview_filename: String,
    pub preview_scroll: u16,
    pub preview_drag_start: Option<(u16, u16)>, // (x, y) for drag start position

    // Blueprint
    pub blueprint_store: BlueprintStore,
    pub selected_blueprint_index: usize,
    pub current_blueprint: Option<Blueprint>,
    pub blueprint_mode: bool,
    pub blueprint_resource_index: usize,
    pub input_buffer: String,
    pub blueprint_markdown_parts: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        let blueprint_store = load_blueprints();
        let settings = load_settings();
        let i18n = I18n::new(settings.language);
        Self {
            screen: Screen::Login,
            running: true,
            loading: false,
            loading_task: LoadingTask::None,
            loading_progress: LoadingProgress::default(),
            last_login_check: None,
            login_info: None,
            login_error: None,
            available_profiles: Vec::new(),
            selected_profile_index: 0,
            selected_region: 0,
            selected_service: 0,
            selected_index: 0,
            message: String::new(),

            settings,
            i18n,
            selected_setting: 0,
            selected_tab: 0,

            instances: Vec::new(),
            vpcs: Vec::new(),
            security_groups: Vec::new(),
            load_balancers: Vec::new(),
            ecr_repositories: Vec::new(),
            auto_scaling_groups: Vec::new(),
            ec2_detail: None,
            network_detail: None,
            sg_detail: None,
            lb_detail: None,
            ecr_detail: None,
            asg_detail: None,

            preview_content: String::new(),
            preview_filename: String::new(),
            preview_scroll: 0,
            preview_drag_start: None,

            blueprint_store,
            selected_blueprint_index: 0,
            current_blueprint: None,
            blueprint_mode: false,
            blueprint_resource_index: 0,
            input_buffer: String::new(),
            blueprint_markdown_parts: Vec::new(),
        }
    }

    pub fn init_auth_flow(&mut self) {
        self.screen = Screen::Login;
        self.last_login_check = None;
        self.refresh_profiles();
    }

    pub fn refresh_profiles(&mut self) {
        match aws_cli::list_aws_profiles() {
            Ok(profiles) => {
                self.available_profiles = profiles;
                self.selected_profile_index = 0;

                if self.available_profiles.is_empty() {
                    self.login_info = None;
                    self.login_error = Some(format!(
                        "{} {}",
                        self.i18n.profile_not_found(),
                        self.i18n.profile_refresh_hint()
                    ));
                    return;
                }

                let preferred_profile = self
                    .settings
                    .aws_profile
                    .clone()
                    .or_else(|| {
                        std::env::var("AWS_PROFILE")
                            .ok()
                            .filter(|value| !value.trim().is_empty())
                    })
                    .or_else(|| {
                        std::env::var("AWS_DEFAULT_PROFILE")
                            .ok()
                            .filter(|value| !value.trim().is_empty())
                    });

                if let Some(profile) = preferred_profile
                    && let Some(index) = self
                        .available_profiles
                        .iter()
                        .position(|item| item == &profile)
                {
                    self.selected_profile_index = index;
                }

                if !self
                    .login_error
                    .as_deref()
                    .is_some_and(|message| is_login_required_message(message, &self.i18n))
                {
                    self.login_error = None;
                }
            }
            Err(err) => {
                self.available_profiles.clear();
                self.selected_profile_index = 0;
                self.login_info = None;
                self.login_error = Some(err);
            }
        }
    }

    pub fn select_current_profile_and_login(&mut self) {
        if self.available_profiles.is_empty() {
            self.refresh_profiles();
            return;
        }

        if self.selected_profile_index >= self.available_profiles.len() {
            self.selected_profile_index = 0;
        }

        let Some(profile) = self
            .available_profiles
            .get(self.selected_profile_index)
            .cloned()
        else {
            return;
        };

        aws_cli::set_aws_profile(&profile);
        self.settings.aws_profile = Some(profile.clone());
        if let Err(error) = save_settings(&self.settings) {
            tracing::warn!(error = %error, profile = %profile, "Failed to persist selected profile");
        }

        self.login_error = None;
        self.check_login();
    }

    pub fn check_login(&mut self) {
        self.message.clear();
        self.last_login_check = Some(Instant::now());
        match aws_cli::check_aws_login() {
            Ok(info) => {
                self.login_info = Some(info);
                self.login_error = None;
                self.screen = Screen::BlueprintSelect;
                tracing::info!("Login check passed; screen moved to BlueprintSelect");
            }
            Err(e) => {
                tracing::warn!(code = ?e.code, detail = %e.detail, "Login check failed");
                if is_login_required_error(&e) {
                    self.login_info = None;
                    self.screen = Screen::Login;
                    self.login_error = Some(auth_error_message(&e, &self.i18n));
                    tracing::warn!("Login required; screen moved to Login");
                } else {
                    self.login_info = None;
                    self.login_error = None;
                    self.screen = Screen::BlueprintSelect;
                    self.message = auth_error_message(&e, &self.i18n);
                    tracing::warn!("Login check degraded (non-auth error); keeping app accessible");
                }
            }
        }
    }

    pub fn validate_login_for_session(&mut self) {
        if self.screen == Screen::Login {
            return;
        }

        let need_check = match self.last_login_check {
            Some(last_checked_at) => last_checked_at.elapsed() >= LOGIN_SESSION_CHECK_INTERVAL,
            None => true,
        };

        if !need_check {
            return;
        }

        self.last_login_check = Some(Instant::now());
        match aws_cli::check_aws_login() {
            Ok(info) => {
                self.login_info = Some(info);
                self.login_error = None;
                if is_aws_credential_message(&self.message, &self.i18n) {
                    self.message.clear();
                }
            }
            Err(e) => {
                tracing::warn!(code = ?e.code, detail = %e.detail, "Session login check failed");
                if is_login_required_error(&e) {
                    self.login_info = None;
                    self.login_error = Some(auth_error_message(&e, &self.i18n));
                    self.screen = Screen::Login;
                    self.refresh_profiles();
                    tracing::warn!(
                        "Session check failed with auth error; returning to profile select"
                    );
                } else {
                    self.message = auth_error_message(&e, &self.i18n);
                    tracing::warn!(
                        "Session check degraded (non-auth error); staying on current screen"
                    );
                }
            }
        }
    }

    pub fn check_login_if_needed_for_current_screen(&mut self) {
        if self.screen == Screen::Login {
            return;
        }
        self.validate_login_for_session();
    }

    pub fn select_region(&mut self) {
        let region = REGIONS[self.selected_region].code;
        aws_cli::set_region(region);
        self.screen = Screen::ServiceSelect;
    }

    pub fn save_file(&mut self) -> Result<(), std::io::Error> {
        crate::output::save_markdown(&self.preview_filename, &self.preview_content)?;
        self.message = format!("{}: {}", self.i18n.save_complete(), self.preview_filename);
        Ok(())
    }

    // Settings methods
    pub fn toggle_language(&mut self) {
        self.settings.language = self.settings.language.toggle();
        self.i18n = I18n::new(self.settings.language);
        self.save_settings();
    }

    pub fn save_settings(&mut self) {
        if save_settings(&self.settings).is_ok() {
            self.message = self.i18n.settings_saved().to_string();
        }
    }

    // Blueprint methods
    pub fn save_blueprints(&mut self) {
        if save_blueprints(&self.blueprint_store).is_err() {
            self.message = self.i18n.blueprint_save_failed().to_string();
        } else {
            self.message = self.i18n.blueprint_saved().to_string();
        }
    }

    pub fn create_blueprint(&mut self, name: String) {
        let blueprint = Blueprint::new(name);
        self.blueprint_store.add_blueprint(blueprint);
        self.selected_blueprint_index = self.blueprint_store.blueprints.len() - 1;
        self.save_blueprints();
    }

    pub fn delete_blueprint(&mut self, index: usize) {
        self.blueprint_store.remove_blueprint(index);
        if self.selected_blueprint_index >= self.blueprint_store.blueprints.len()
            && self.selected_blueprint_index > 0
        {
            self.selected_blueprint_index -= 1;
        }
        self.save_blueprints();
        self.message = self.i18n.blueprint_deleted().to_string();
    }

    pub fn add_resource_to_current_blueprint(&mut self, resource: BlueprintResource) {
        if let Some(ref mut blueprint) = self.current_blueprint {
            blueprint.add_resource(resource);
            // Update in store
            if let Some(stored) = self
                .blueprint_store
                .get_blueprint_mut(self.selected_blueprint_index)
            {
                *stored = blueprint.clone();
            }
            self.save_blueprints();
            self.message = self.i18n.resource_added().to_string();
        }
    }

    pub fn remove_resource_from_current_blueprint(&mut self, index: usize) {
        if let Some(ref mut blueprint) = self.current_blueprint {
            blueprint.remove_resource(index);
            // Update in store
            if let Some(stored) = self
                .blueprint_store
                .get_blueprint_mut(self.selected_blueprint_index)
            {
                *stored = blueprint.clone();
            }
            self.save_blueprints();
            self.message = self.i18n.resource_deleted().to_string();
        }
    }

    pub fn move_resource_up(&mut self, index: usize) -> bool {
        if index == 0 {
            return false;
        }
        if let Some(ref mut blueprint) = self.current_blueprint
            && index < blueprint.resources.len()
        {
            blueprint.resources.swap(index, index - 1);
            // Update in store
            if let Some(stored) = self
                .blueprint_store
                .get_blueprint_mut(self.selected_blueprint_index)
            {
                *stored = blueprint.clone();
            }
            self.save_blueprints();
            return true;
        }
        false
    }

    pub fn move_resource_down(&mut self, index: usize) -> bool {
        if let Some(ref mut blueprint) = self.current_blueprint
            && index + 1 < blueprint.resources.len()
        {
            blueprint.resources.swap(index, index + 1);
            // Update in store
            if let Some(stored) = self
                .blueprint_store
                .get_blueprint_mut(self.selected_blueprint_index)
            {
                *stored = blueprint.clone();
            }
            self.save_blueprints();
            return true;
        }
        false
    }

    pub fn get_current_resource_type(&self) -> Option<ResourceType> {
        if self.ec2_detail.is_some() {
            Some(ResourceType::Ec2)
        } else if self.network_detail.is_some() {
            Some(ResourceType::Network)
        } else if self.sg_detail.is_some() {
            Some(ResourceType::SecurityGroup)
        } else if self.lb_detail.is_some() {
            Some(ResourceType::LoadBalancer)
        } else if self.ecr_detail.is_some() {
            Some(ResourceType::Ecr)
        } else if self.asg_detail.is_some() {
            Some(ResourceType::Asg)
        } else {
            None
        }
    }

    pub fn get_current_resource_info(&self) -> Option<(String, String)> {
        if let Some(ref detail) = self.ec2_detail {
            Some((detail.instance_id.clone(), detail.name.clone()))
        } else if let Some(ref detail) = self.network_detail {
            Some((detail.id.clone(), detail.name.clone()))
        } else if let Some(ref detail) = self.sg_detail {
            Some((detail.id.clone(), detail.name.clone()))
        } else if let Some(ref detail) = self.lb_detail {
            Some((detail.arn.clone(), detail.name.clone()))
        } else if let Some(ref detail) = self.ecr_detail {
            Some((detail.name.clone(), detail.name.clone()))
        } else {
            self.asg_detail
                .as_ref()
                .map(|detail| (detail.name.clone(), detail.name.clone()))
        }
    }

    pub fn get_current_region(&self) -> String {
        REGIONS[self.selected_region].code.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{App, LoadingProgress, REGIONS, Region};
    use crate::aws_cli::iam::{AttachedPolicy, IamRoleDetail, InlinePolicy};
    use crate::aws_cli::{
        AsgDetail, AwsAuthError, AwsAuthErrorCode, Ec2Detail, EcrDetail, EipDetail,
        LoadBalancerDetail, NatDetail, NetworkDetail, RouteTableDetail, ScalingPolicy,
        SecurityGroupDetail, SecurityRule, TargetGroupInfo,
    };
    use crate::blueprint::{Blueprint, BlueprintResource, ResourceType};
    use crate::i18n::{I18n, Language};
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn temp_home(prefix: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        path.push(format!(
            "emd-app-{}-{}-{}",
            prefix,
            std::process::id(),
            nanos
        ));
        path
    }

    fn restore_var(name: &str, value: Option<OsString>) {
        if let Some(v) = value {
            unsafe {
                env::set_var(name, v);
            }
        } else {
            unsafe {
                env::remove_var(name);
            }
        }
    }

    fn sample_ec2_detail() -> Ec2Detail {
        Ec2Detail {
            name: "web-a".to_string(),
            instance_id: "i-0123456789abcdef0".to_string(),
            instance_type: "t3.micro".to_string(),
            ami: "ubuntu-22.04".to_string(),
            platform: "Linux".to_string(),
            architecture: "x86_64".to_string(),
            key_pair: "main-key".to_string(),
            vpc: "vpc-main".to_string(),
            subnet: "subnet-a".to_string(),
            az: "ap-northeast-2a".to_string(),
            public_ip: "1.1.1.1".to_string(),
            private_ip: "10.0.0.10".to_string(),
            security_groups: vec!["sg-web".to_string()],
            state: "running".to_string(),
            ebs_optimized: true,
            monitoring: "Enabled".to_string(),
            iam_role: Some("role-web".to_string()),
            iam_role_detail: Some(IamRoleDetail {
                name: "role-web".to_string(),
                arn: "arn:aws:iam::123456789012:role/role-web".to_string(),
                assume_role_policy: "{}".to_string(),
                attached_policies: vec![AttachedPolicy {
                    name: "ReadOnlyAccess".to_string(),
                    arn: "arn:aws:iam::aws:policy/ReadOnlyAccess".to_string(),
                }],
                inline_policies: vec![InlinePolicy {
                    name: "inline-policy".to_string(),
                    document: "{}".to_string(),
                }],
            }),
            launch_time: "2026-02-13".to_string(),
            tags: vec![("Name".to_string(), "web-a".to_string())],
            volumes: vec![],
            user_data: None,
        }
    }

    fn sample_network_detail() -> NetworkDetail {
        NetworkDetail {
            name: "main-vpc".to_string(),
            id: "vpc-1111aaaa".to_string(),
            cidr: "10.0.0.0/16".to_string(),
            state: "available".to_string(),
            subnets: vec![],
            igws: vec![],
            nats: vec![NatDetail {
                name: "nat-a".to_string(),
                id: "nat-1234".to_string(),
                state: "available".to_string(),
                connectivity_type: "public".to_string(),
                availability_mode: "regional".to_string(),
                auto_scaling_ips: "enabled".to_string(),
                auto_provision_zones: "enabled".to_string(),
                public_ip: "1.1.1.1".to_string(),
                allocation_id: "eipalloc-1234".to_string(),
                subnet_id: "subnet-a".to_string(),
                tags: vec![],
            }],
            route_tables: vec![RouteTableDetail {
                name: "rt-main".to_string(),
                id: "rtb-1234".to_string(),
                routes: vec![],
                associations: vec![],
            }],
            eips: vec![EipDetail {
                name: "eip-a".to_string(),
                public_ip: "1.1.1.1".to_string(),
                instance_id: String::new(),
                private_ip: String::new(),
            }],
            dns_support: true,
            dns_hostnames: true,
            tags: vec![("Name".to_string(), "main-vpc".to_string())],
        }
    }

    fn sample_sg_detail() -> SecurityGroupDetail {
        SecurityGroupDetail {
            name: "sg-web".to_string(),
            id: "sg-1234".to_string(),
            description: "web sg".to_string(),
            vpc_id: "vpc-1111aaaa".to_string(),
            inbound_rules: vec![SecurityRule {
                protocol: "TCP".to_string(),
                port_range: "80".to_string(),
                source_dest: "0.0.0.0/0".to_string(),
                description: "-".to_string(),
            }],
            outbound_rules: vec![],
        }
    }

    fn sample_lb_detail() -> LoadBalancerDetail {
        LoadBalancerDetail {
            name: "alb-main".to_string(),
            arn: "arn:aws:elasticloadbalancing:ap-northeast-2:123456789012:loadbalancer/app/alb-main/1234".to_string(),
            dns_name: "alb-main.example.com".to_string(),
            lb_type: "application".to_string(),
            scheme: "internet-facing".to_string(),
            vpc_id: "vpc-1111aaaa".to_string(),
            ip_address_type: "ipv4".to_string(),
            state: "active".to_string(),
            availability_zones: vec!["ap-northeast-2a".to_string()],
            security_groups: vec!["sg-1234".to_string()],
            listeners: vec![],
            target_groups: vec![TargetGroupInfo {
                name: "tg-main".to_string(),
                arn: "arn:aws:elasticloadbalancing:ap-northeast-2:123456789012:targetgroup/tg-main/1234".to_string(),
                protocol: "HTTP".to_string(),
                port: 80,
                target_type: "instance".to_string(),
                health_check_protocol: "HTTP".to_string(),
                health_check_path: "/health".to_string(),
                healthy_threshold: 2,
                unhealthy_threshold: 3,
                targets: vec![],
            }],
        }
    }

    fn sample_ecr_detail() -> EcrDetail {
        EcrDetail {
            name: "repo-a".to_string(),
            uri: "123456789012.dkr.ecr.ap-northeast-2.amazonaws.com/repo-a".to_string(),
            tag_mutability: "MUTABLE".to_string(),
            encryption_type: "AES256".to_string(),
            kms_key: None,
            created_at: "2026-02-13".to_string(),
            image_count: 2,
        }
    }

    fn sample_asg_detail() -> AsgDetail {
        AsgDetail {
            name: "asg-main".to_string(),
            arn: "arn:aws:autoscaling:ap-northeast-2:123456789012:autoScalingGroup:abcd:autoScalingGroupName/asg-main".to_string(),
            launch_template_name: Some("lt-main".to_string()),
            launch_template_id: Some("lt-1234".to_string()),
            launch_config_name: None,
            min_size: 1,
            max_size: 3,
            desired_capacity: 2,
            default_cooldown: 300,
            availability_zones: vec!["ap-northeast-2a".to_string()],
            target_group_arns: vec![],
            health_check_type: "EC2".to_string(),
            health_check_grace_period: 120,
            instances: vec![],
            created_time: "2026-02-13".to_string(),
            scaling_policies: vec![ScalingPolicy {
                name: "scale-out".to_string(),
                policy_type: "SimpleScaling".to_string(),
                adjustment_type: Some("ChangeInCapacity".to_string()),
                scaling_adjustment: Some(1),
                cooldown: Some(60),
            }],
            tags: vec![],
        }
    }

    #[test]
    fn loading_progress_reset_works() {
        let mut p = LoadingProgress {
            vpc_info: true,
            subnets: true,
            igws: true,
            nats: true,
            route_tables: true,
            eips: true,
            dns_attrs: true,
        };
        p.reset();
        assert!(!p.vpc_info);
        assert!(!p.subnets);
        assert!(!p.igws);
        assert!(!p.nats);
        assert!(!p.route_tables);
        assert!(!p.eips);
        assert!(!p.dns_attrs);
    }

    #[test]
    fn login_required_error_detector_matches_auth_codes() {
        assert!(super::is_login_required_error(&AwsAuthError {
            code: AwsAuthErrorCode::CredentialsLoadFailed,
            detail: "fail".to_string(),
        }));
        assert!(super::is_login_required_error(&AwsAuthError {
            code: AwsAuthErrorCode::CallerIdentityFailed,
            detail: "fail".to_string(),
        }));
        assert!(!super::is_login_required_error(&AwsAuthError {
            code: AwsAuthErrorCode::Network,
            detail: "network".to_string(),
        }));
    }

    #[test]
    fn aws_credential_message_detector_matches_expected_messages() {
        let i18n = I18n::new(crate::i18n::Language::English);
        assert!(super::is_aws_credential_message(
            i18n.auth_network_error(),
            &i18n
        ));
        assert!(super::is_aws_credential_message(
            i18n.auth_unknown_error(),
            &i18n
        ));
        assert!(!super::is_aws_credential_message(
            "Save complete: output.md",
            &i18n
        ));
    }

    #[test]
    fn auth_error_message_maps_each_auth_code_to_i18n_text() {
        let i18n = I18n::new(Language::English);
        assert_eq!(
            super::auth_error_message(
                &AwsAuthError {
                    code: AwsAuthErrorCode::CredentialsProviderMissing,
                    detail: "x".to_string(),
                },
                &i18n
            ),
            i18n.auth_provider_missing()
        );
        assert_eq!(
            super::auth_error_message(
                &AwsAuthError {
                    code: AwsAuthErrorCode::CredentialsLoadFailed,
                    detail: "x".to_string(),
                },
                &i18n
            ),
            i18n.auth_credentials_load_failed()
        );
        assert_eq!(
            super::auth_error_message(
                &AwsAuthError {
                    code: AwsAuthErrorCode::CallerIdentityFailed,
                    detail: "x".to_string(),
                },
                &i18n
            ),
            i18n.auth_caller_identity_failed()
        );
        assert_eq!(
            super::auth_error_message(
                &AwsAuthError {
                    code: AwsAuthErrorCode::Network,
                    detail: "x".to_string(),
                },
                &i18n
            ),
            i18n.auth_network_error()
        );
        assert_eq!(
            super::auth_error_message(
                &AwsAuthError {
                    code: AwsAuthErrorCode::Unknown,
                    detail: "x".to_string(),
                },
                &i18n
            ),
            i18n.auth_unknown_error()
        );
    }

    #[test]
    fn refresh_profiles_selects_saved_profile_when_present() {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let original_home = env::var_os("HOME");
        let home = temp_home("profiles");
        let aws_dir = home.join(".aws");
        fs::create_dir_all(&aws_dir).expect("create .aws");
        fs::write(
            aws_dir.join("config"),
            "[default]\nregion=ap-southeast-1\n[profile dev]\n",
        )
        .expect("write config");
        fs::write(
            aws_dir.join("credentials"),
            "[default]\naws_access_key_id=x\n[ops]\naws_access_key_id=y\n",
        )
        .expect("write credentials");

        unsafe {
            env::set_var("HOME", &home);
        }

        let mut app = App::new();
        app.settings.aws_profile = Some("dev".to_string());
        app.refresh_profiles();

        assert_eq!(app.available_profiles, vec!["default", "dev", "ops"]);
        assert_eq!(app.selected_profile_index, 1);
        assert!(app.login_error.is_none());

        restore_var("HOME", original_home);
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn refresh_profiles_sets_error_when_no_profile_files_exist() {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let original_home = env::var_os("HOME");
        let home = temp_home("no-profiles");
        fs::create_dir_all(&home).expect("create home");

        unsafe {
            env::set_var("HOME", &home);
        }

        let mut app = App::new();
        app.refresh_profiles();

        assert!(app.available_profiles.is_empty());
        assert!(
            app.login_error
                .as_deref()
                .unwrap_or_default()
                .contains(app.i18n.profile_not_found())
        );

        restore_var("HOME", original_home);
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn region_name_uses_language() {
        let r = Region {
            code: "us-east-1",
            name_ko: "버지니아",
            name_en: "N. Virginia",
        };
        assert_eq!(r.name(crate::i18n::Language::Korean), "버지니아");
        assert_eq!(r.name(crate::i18n::Language::English), "N. Virginia");
    }

    #[test]
    fn get_current_resource_type_and_info_priority() {
        let mut app = App::new();

        app.ec2_detail = Some(sample_ec2_detail());
        assert_eq!(app.get_current_resource_type(), Some(ResourceType::Ec2));
        assert_eq!(
            app.get_current_resource_info(),
            Some(("i-0123456789abcdef0".to_string(), "web-a".to_string()))
        );

        app.ec2_detail = None;
        app.network_detail = Some(sample_network_detail());
        assert_eq!(app.get_current_resource_type(), Some(ResourceType::Network));

        app.network_detail = None;
        app.sg_detail = Some(sample_sg_detail());
        assert_eq!(
            app.get_current_resource_type(),
            Some(ResourceType::SecurityGroup)
        );

        app.sg_detail = None;
        app.lb_detail = Some(sample_lb_detail());
        assert_eq!(
            app.get_current_resource_type(),
            Some(ResourceType::LoadBalancer)
        );

        app.lb_detail = None;
        app.ecr_detail = Some(sample_ecr_detail());
        assert_eq!(app.get_current_resource_type(), Some(ResourceType::Ecr));

        app.ecr_detail = None;
        app.asg_detail = Some(sample_asg_detail());
        assert_eq!(app.get_current_resource_type(), Some(ResourceType::Asg));
    }

    #[test]
    fn get_current_region_returns_selected_code() {
        let mut app = App::new();
        app.selected_region = REGIONS.len().saturating_sub(1);
        assert_eq!(app.get_current_region(), REGIONS[REGIONS.len() - 1].code);
    }

    #[test]
    fn move_resource_up_down_respects_bounds() {
        let mut app = App::new();
        let mut blueprint = Blueprint::new("bp".to_string());
        blueprint.add_resource(BlueprintResource {
            resource_type: ResourceType::Ec2,
            region: "ap-northeast-2".to_string(),
            resource_id: "i-1".to_string(),
            resource_name: "web-1".to_string(),
        });
        blueprint.add_resource(BlueprintResource {
            resource_type: ResourceType::Ec2,
            region: "ap-northeast-2".to_string(),
            resource_id: "i-2".to_string(),
            resource_name: "web-2".to_string(),
        });

        app.current_blueprint = Some(blueprint.clone());
        app.blueprint_store.add_blueprint(blueprint);
        app.selected_blueprint_index = 0;

        assert!(!app.move_resource_up(0));
        assert!(app.move_resource_down(0));
        assert!(!app.move_resource_down(99));
    }

    #[test]
    fn toggle_language_updates_settings_and_i18n() {
        let mut app = App::new();
        let before = app.settings.language;
        app.toggle_language();
        assert_ne!(app.settings.language, before);
        assert_eq!(app.i18n.lang, app.settings.language);
    }

    #[test]
    fn add_and_remove_resource_on_current_blueprint() {
        let mut app = App::new();
        let blueprint = Blueprint::new("bp".to_string());
        app.blueprint_store.add_blueprint(blueprint.clone());
        app.current_blueprint = Some(blueprint);
        app.selected_blueprint_index = 0;

        app.add_resource_to_current_blueprint(BlueprintResource {
            resource_type: ResourceType::Ecr,
            region: "ap-northeast-2".to_string(),
            resource_id: "repo-a".to_string(),
            resource_name: "repo-a".to_string(),
        });
        assert_eq!(
            app.current_blueprint
                .as_ref()
                .map(|bp| bp.resources.len())
                .unwrap_or_default(),
            1
        );

        app.remove_resource_from_current_blueprint(0);
        assert_eq!(
            app.current_blueprint
                .as_ref()
                .map(|bp| bp.resources.len())
                .unwrap_or_default(),
            0
        );
    }
}
