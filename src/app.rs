use crate::aws_cli::{self, AsgDetail, AwsResource, Ec2Detail, EcrDetail};
use crate::blueprint::{
    Blueprint, BlueprintResource, BlueprintStore, ResourceType, load_blueprints, save_blueprints,
};
use crate::i18n::{I18n, Language};
use crate::settings::{AppSettings, load_settings, save_settings};

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

#[allow(dead_code)]
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
    pub login_info: Option<String>,
    pub login_error: Option<String>,
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
            login_info: None,
            login_error: None,
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

    pub fn check_login(&mut self) {
        match aws_cli::check_aws_login() {
            Ok(info) => {
                self.login_info = Some(info);
                self.screen = Screen::BlueprintSelect;
            }
            Err(e) => {
                self.login_error = Some(e);
            }
        }
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
            self.message = "Save failed".to_string();
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
