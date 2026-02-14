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

#[cfg(test)]
mod tests {
    use super::{Blueprint, BlueprintResource, BlueprintStore, ResourceType};

    fn sample_resource(resource_type: ResourceType, suffix: &str) -> BlueprintResource {
        BlueprintResource {
            resource_type,
            region: "ap-northeast-2".to_string(),
            resource_id: format!("id-{}", suffix),
            resource_name: format!("name-{}", suffix),
        }
    }

    #[test]
    fn resource_type_display_covers_all_variants() {
        assert_eq!(ResourceType::Ec2.display(), "EC2");
        assert_eq!(ResourceType::Network.display(), "Network");
        assert_eq!(ResourceType::SecurityGroup.display(), "Security Group");
        assert_eq!(ResourceType::LoadBalancer.display(), "Load Balancer");
        assert_eq!(ResourceType::Ecr.display(), "ECR");
        assert_eq!(ResourceType::Asg.display(), "Auto Scaling Group");
    }

    #[test]
    fn blueprint_resource_display_contains_type_name_id_and_region() {
        let resource = sample_resource(ResourceType::Ec2, "abc");
        let text = resource.display();
        assert!(text.contains("[EC2]"));
        assert!(text.contains("name-abc"));
        assert!(text.contains("id-abc"));
        assert!(text.contains("ap-northeast-2"));
    }

    #[test]
    fn blueprint_add_resource_deduplicates_same_id_and_region() {
        let mut blueprint = Blueprint::new("bp".to_string());
        let resource = sample_resource(ResourceType::Ec2, "dup");

        blueprint.add_resource(resource.clone());
        blueprint.add_resource(resource);

        assert_eq!(blueprint.resources.len(), 1);
    }

    #[test]
    fn blueprint_remove_resource_ignores_out_of_bounds() {
        let mut blueprint = Blueprint::new("bp".to_string());
        blueprint.add_resource(sample_resource(ResourceType::Ecr, "1"));
        blueprint.remove_resource(99);
        assert_eq!(blueprint.resources.len(), 1);
    }

    #[test]
    fn blueprint_store_add_get_remove_works() {
        let mut store = BlueprintStore::new();
        store.add_blueprint(Blueprint::new("first".to_string()));
        store.add_blueprint(Blueprint::new("second".to_string()));

        assert_eq!(store.blueprints.len(), 2);
        assert_eq!(
            store.get_blueprint(0).map(|b| b.name.as_str()),
            Some("first")
        );

        if let Some(bp) = store.get_blueprint_mut(1) {
            bp.name = "second-updated".to_string();
        }
        assert_eq!(
            store.get_blueprint(1).map(|b| b.name.as_str()),
            Some("second-updated")
        );

        store.remove_blueprint(0);
        assert_eq!(store.blueprints.len(), 1);
        assert_eq!(
            store.get_blueprint(0).map(|b| b.name.as_str()),
            Some("second-updated")
        );
    }
}
