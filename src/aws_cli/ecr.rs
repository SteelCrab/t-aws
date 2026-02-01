use crate::aws_cli::common::{AwsResource, run_aws_cli};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct EcrRepositoriesResponse {
    repositories: Vec<EcrRepository>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EcrRepository {
    repository_name: String,
    repository_uri: String,
    #[serde(default)]
    image_tag_mutability: String,
    #[serde(default)]
    encryption_configuration: Option<EcrEncryptionConfiguration>,
    #[serde(default)]
    created_at: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EcrEncryptionConfiguration {
    encryption_type: String,
    #[serde(default)]
    kms_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EcrImagesResponse {
    image_details: Vec<EcrImageDetail>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EcrImageDetail {
    #[allow(dead_code)]
    image_digest: String,
}

#[derive(Debug)]
pub struct EcrDetail {
    pub name: String,
    pub uri: String,
    pub tag_mutability: String,
    pub encryption_type: String,
    pub kms_key: Option<String>,
    pub created_at: String,
    pub image_count: i32,
}

impl EcrDetail {
    pub fn to_markdown(&self) -> String {
        let encryption_display = if self.encryption_type == "KMS" {
            if let Some(ref key) = self.kms_key {
                format!("AWS KMS ({})", key)
            } else {
                "AWS KMS".to_string()
            }
        } else {
            "AES-256".to_string()
        };

        let lines = vec![
            format!("## ECR 레포지토리 ({})\n", self.name),
            "| 항목 | 값 |".to_string(),
            "|:---|:---|".to_string(),
            format!("| 이름 | {} |", self.name),
            format!("| URI | {} |", self.uri),
            format!("| 태그 변경 가능 | {} |", self.tag_mutability),
            format!("| 암호화 | {} |", encryption_display),
            format!("| 이미지 수 | {} |", self.image_count),
            format!("| 생성일 | {} |", self.created_at),
        ];

        lines.join("\n") + "\n"
    }
}

pub fn list_ecr_repositories() -> Vec<AwsResource> {
    let output = match run_aws_cli(&["ecr", "describe-repositories", "--output", "json"]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let response: EcrRepositoriesResponse = match serde_json::from_str(&output) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    response
        .repositories
        .into_iter()
        .map(|repo| {
            let mutability = if repo.image_tag_mutability == "IMMUTABLE" {
                "Immutable"
            } else {
                "Mutable"
            };

            AwsResource {
                name: format!("{} ({})", repo.repository_name, mutability),
                id: repo.repository_name,
                state: repo.image_tag_mutability,
                az: String::new(),
                cidr: repo.repository_uri,
            }
        })
        .collect()
}

pub fn get_ecr_detail(repo_name: &str) -> Option<EcrDetail> {
    let output = run_aws_cli(&[
        "ecr",
        "describe-repositories",
        "--repository-names",
        repo_name,
        "--output",
        "json",
    ])?;

    let response: EcrRepositoriesResponse = serde_json::from_str(&output).ok()?;
    let repo = response.repositories.first()?;

    let images_output = run_aws_cli(&[
        "ecr",
        "describe-images",
        "--repository-name",
        repo_name,
        "--output",
        "json",
    ]);

    let image_count = images_output
        .and_then(|o| serde_json::from_str::<EcrImagesResponse>(&o).ok())
        .map(|r| r.image_details.len() as i32)
        .unwrap_or(0);

    let created_at = repo
        .created_at
        .map(|ts| {
            let secs = ts as i64;
            chrono::DateTime::from_timestamp(secs, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "-".to_string())
        })
        .unwrap_or_else(|| "-".to_string());

    let (encryption_type, kms_key) = repo
        .encryption_configuration
        .as_ref()
        .map(|enc| (enc.encryption_type.clone(), enc.kms_key.clone()))
        .unwrap_or_else(|| ("AES256".to_string(), None));

    Some(EcrDetail {
        name: repo.repository_name.clone(),
        uri: repo.repository_uri.clone(),
        tag_mutability: repo.image_tag_mutability.clone(),
        encryption_type,
        kms_key,
        created_at,
        image_count,
    })
}
