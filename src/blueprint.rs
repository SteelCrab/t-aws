use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    Ec2,
    Network,
    SecurityGroup,
    LoadBalancer,
    Ecr,
    Asg,
}

impl ResourceType {
    pub fn display(&self) -> &'static str {
        match self {
            ResourceType::Ec2 => "EC2",
            ResourceType::Network => "Network",
            ResourceType::SecurityGroup => "Security Group",
            ResourceType::LoadBalancer => "Load Balancer",
            ResourceType::Ecr => "ECR",
            ResourceType::Asg => "Auto Scaling Group",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintResource {
    pub resource_type: ResourceType,
    pub region: String,
    pub resource_id: String,
    pub resource_name: String,
}

impl BlueprintResource {
    pub fn display(&self) -> String {
        format!(
            "[{}] {} - {} ({})",
            self.resource_type.display(),
            self.resource_name,
            self.resource_id,
            self.region
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub id: String,
    pub name: String,
    pub resources: Vec<BlueprintResource>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Blueprint {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            resources: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_resource(&mut self, resource: BlueprintResource) {
        // Check for duplicates
        let exists = self
            .resources
            .iter()
            .any(|r| r.resource_id == resource.resource_id && r.region == resource.region);
        if !exists {
            self.resources.push(resource);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_resource(&mut self, index: usize) {
        if index < self.resources.len() {
            self.resources.remove(index);
            self.updated_at = Utc::now();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlueprintStore {
    pub blueprints: Vec<Blueprint>,
}

impl BlueprintStore {
    pub fn new() -> Self {
        Self {
            blueprints: Vec::new(),
        }
    }

    pub fn add_blueprint(&mut self, blueprint: Blueprint) {
        self.blueprints.push(blueprint);
    }

    pub fn remove_blueprint(&mut self, index: usize) {
        if index < self.blueprints.len() {
            self.blueprints.remove(index);
        }
    }

    pub fn get_blueprint(&self, index: usize) -> Option<&Blueprint> {
        self.blueprints.get(index)
    }

    pub fn get_blueprint_mut(&mut self, index: usize) -> Option<&mut Blueprint> {
        self.blueprints.get_mut(index)
    }
}

fn get_blueprint_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let emd_dir = home.join(".emd");

    // Create directory if it doesn't exist
    if !emd_dir.exists() {
        fs::create_dir_all(&emd_dir).ok()?;
    }

    Some(emd_dir.join("blueprints.json"))
}

pub fn load_blueprints() -> BlueprintStore {
    let path = match get_blueprint_path() {
        Some(p) => p,
        None => return BlueprintStore::new(),
    };

    if !path.exists() {
        return BlueprintStore::new();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| BlueprintStore::new()),
        Err(_) => BlueprintStore::new(),
    }
}

pub fn save_blueprints(store: &BlueprintStore) -> Result<(), std::io::Error> {
    let path = get_blueprint_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
    })?;

    let content = serde_json::to_string_pretty(store)?;
    fs::write(&path, content)?;
    Ok(())
}
