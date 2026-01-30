use crate::aws_cli::{self, AwsResource, Ec2Detail};
use crate::blueprint::{
    Blueprint, BlueprintResource, BlueprintStore, ResourceType, load_blueprints, save_blueprints,
};

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
    Preview,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadingTask {
    None,
    RefreshEc2,
    RefreshVpc,
    RefreshPreview,
    RefreshSecurityGroup,
    RefreshLoadBalancer,
    LoadEc2,
    LoadVpc,
    LoadSecurityGroup,
    LoadLoadBalancer,
    LoadEc2Detail(String),
    LoadVpcDetail(String, u8), // (vpc_id, step: 0-6)
    LoadSecurityGroupDetail(String),
    LoadLoadBalancerDetail(String),
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
    pub name: &'static str,
}

pub const REGIONS: &[Region] = &[
    Region {
        code: "ap-northeast-2",
        name: "서울",
    },
    Region {
        code: "ap-northeast-1",
        name: "도쿄",
    },
    Region {
        code: "ap-northeast-3",
        name: "오사카",
    },
    Region {
        code: "ap-southeast-1",
        name: "싱가포르",
    },
    Region {
        code: "ap-southeast-2",
        name: "시드니",
    },
    Region {
        code: "ap-south-1",
        name: "뭄바이",
    },
    Region {
        code: "us-east-1",
        name: "버지니아",
    },
    Region {
        code: "us-east-2",
        name: "오하이오",
    },
    Region {
        code: "us-west-1",
        name: "캘리포니아",
    },
    Region {
        code: "us-west-2",
        name: "오레곤",
    },
    Region {
        code: "eu-west-1",
        name: "아일랜드",
    },
    Region {
        code: "eu-central-1",
        name: "프랑크푸르트",
    },
];

pub const SERVICES: &[&str] = &["EC2", "Network", "Security Group", "Load Balancer", "종료"];

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

    // AWS Resources
    pub instances: Vec<AwsResource>,
    pub vpcs: Vec<AwsResource>,
    pub security_groups: Vec<AwsResource>,
    pub load_balancers: Vec<AwsResource>,

    // Selected EC2 Detail
    pub ec2_detail: Option<Ec2Detail>,
    // Selected Network Detail
    pub network_detail: Option<aws_cli::NetworkDetail>,
    // Selected Security Group Detail
    pub sg_detail: Option<aws_cli::SecurityGroupDetail>,
    // Selected Load Balancer Detail
    pub lb_detail: Option<aws_cli::LoadBalancerDetail>,

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

            instances: Vec::new(),
            vpcs: Vec::new(),
            security_groups: Vec::new(),
            load_balancers: Vec::new(),
            ec2_detail: None,
            network_detail: None,
            sg_detail: None,
            lb_detail: None,

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
        self.message = format!("저장 완료: {}", self.preview_filename);
        Ok(())
    }

    // Blueprint methods
    pub fn save_blueprints(&mut self) {
        if let Err(e) = save_blueprints(&self.blueprint_store) {
            self.message = format!("저장 실패: {}", e);
        } else {
            self.message = "블루프린터 저장 완료".to_string();
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
        self.message = "블루프린터 삭제 완료".to_string();
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
            self.message = "리소스 추가 완료".to_string();
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
            self.message = "리소스 삭제 완료".to_string();
        }
    }

    pub fn move_resource_up(&mut self, index: usize) -> bool {
        if index == 0 {
            return false;
        }
        if let Some(ref mut blueprint) = self.current_blueprint
            && index < blueprint.resources.len() {
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
            && index + 1 < blueprint.resources.len() {
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
        } else { self.lb_detail.as_ref().map(|detail| (detail.arn.clone(), detail.name.clone())) }
    }

    pub fn get_current_region(&self) -> String {
        REGIONS[self.selected_region].code.to_string()
    }
}
