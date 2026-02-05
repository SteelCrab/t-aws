use crate::aws_cli::common::{AwsResource, get_runtime, get_sdk_config};
use crate::i18n::{I18n, Language};
use aws_config::BehaviorVersion;
use aws_sdk_ecr::types::ImageDetail;
use chrono::DateTime;
use serde::Deserialize;

#[derive(Debug)]
pub struct EcrImageInfo {
    pub tag: String,
    pub digest: String,
    pub size_mb: f64,
    pub pushed_at: String,
    pub scan_status: String,
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
    pub images: Vec<EcrImageInfo>,
}

impl EcrDetail {
    pub fn to_markdown(&self, lang: Language) -> String {
        let i18n = I18n::new(lang);
        let encryption_display = if self.encryption_type == "KMS" {
            if let Some(ref key) = self.kms_key {
                format!("AWS KMS ({})", key)
            } else {
                "AWS KMS".to_string()
            }
        } else {
            "AES-256".to_string()
        };

        let mut lines = vec![
            format!("## {} ({})\n", i18n.md_ecr_repository(), self.name),
            format!("| {} | {} |", i18n.item(), i18n.value()),
            "|:---|:---|".to_string(),
            format!("| {} | {} |", i18n.md_name(), self.name),
            format!("| URI | {} |", self.uri),
            format!("| {} | {} |", i18n.md_tag_mutability(), self.tag_mutability),
            format!("| {} | {} |", i18n.md_encryption(), encryption_display),
            format!("| {} | {} |", i18n.md_image_count(), self.image_count),
            format!("| {} | {} |", i18n.md_created_at(), self.created_at),
        ];

        if !self.images.is_empty() {
            lines.push("\n### 이미지 목록\n".to_string());
            lines.push("| 태그 | 크기 (MB) | 푸시 날짜 | 스캔 상태 |".to_string());
            lines.push("|:---|---:|:---|:---|".to_string());

            let mut images = self.images.iter().collect::<Vec<_>>();
            images.sort_by(|a, b| b.pushed_at.cmp(&a.pushed_at));

            for img in images.iter().take(20) {
                lines.push(format!(
                    "| {} | {:.2} | {} | {} |",
                    img.tag, img.size_mb, img.pushed_at, img.scan_status
                ));
            }
        }

        lines.join("\n") + "\n"
    }
}

pub fn list_ecr_repositories() -> Vec<AwsResource> {
    let rt = get_runtime();
    let config = get_sdk_config();
    let client = aws_sdk_ecr::Client::new(&config);

    rt.block_on(async {
        match client.describe_repositories().send().await {
            Ok(output) => output
                .repositories()
                .iter()
                .map(|repo| {
                    let name = repo.repository_name().unwrap_or("").to_string();
                    let uri = repo.repository_uri().unwrap_or("").to_string();
                    let mutability_raw = repo
                        .image_tag_mutability()
                        .map(|m| m.as_str())
                        .unwrap_or("UNKNOWN");

                    let mutability = if mutability_raw == "IMMUTABLE" {
                        "Immutable"
                    } else {
                        "Mutable"
                    };

                    AwsResource {
                        name: format!("{} · {} · {}", name, mutability, name),
                        id: name,
                        state: mutability_raw.to_string(),
                        az: String::new(),
                        cidr: uri,
                    }
                })
                .collect(),
            Err(e) => {
                eprintln!("Error listing ECR repositories: {:?}", e);
                Vec::new()
            }
        }
    })
}

pub fn get_ecr_detail(repo_name: &str) -> Option<EcrDetail> {
    let rt = get_runtime();
    let config = get_sdk_config();
    let client = aws_sdk_ecr::Client::new(&config);

    rt.block_on(async {
        // 1. 레포지토리 정보 조회
        let repo_output = client
            .describe_repositories()
            .repository_names(repo_name)
            .send()
            .await
            .ok()?;

        let repo = repo_output.repositories().first()?;

        // 2. 이미지 목록 조회
        let images_output = client
            .describe_images()
            .repository_name(repo_name)
            .send()
            .await
            .ok();

        let mut images = Vec::new();
        let mut image_count = 0;

        if let Some(output) = images_output {
            image_count = output.image_details().len() as i32;

            for img in output.image_details() {
                let tags = img.image_tags();
                for tag in tags {
                    let size_mb = img.image_size_in_bytes().unwrap_or(0) as f64 / 1024.0 / 1024.0;
                    let pushed_at = img
                        .image_pushed_at()
                        .map(|ts| {
                            let secs = ts.secs() as i64;
                            DateTime::from_timestamp(secs, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                .unwrap_or_else(|| "-".to_string())
                        })
                        .unwrap_or_else(|| "-".to_string());

                    let scan_status = img
                        .image_scan_findings_summary()
                        .map(|s| {
                            let mut parts = Vec::new();
                            if let Some(counts) = s.finding_severity_counts() {
                                if let Some(&cnt) =
                                    counts.get(&aws_sdk_ecr::types::FindingSeverity::Critical)
                                {
                                    if cnt > 0 {
                                        parts.push(format!("CRITICAL:{}", cnt));
                                    }
                                }
                                if let Some(&cnt) =
                                    counts.get(&aws_sdk_ecr::types::FindingSeverity::High)
                                {
                                    if cnt > 0 {
                                        parts.push(format!("HIGH:{}", cnt));
                                    }
                                }
                            }
                            if parts.is_empty() {
                                "Passed".to_string()
                            } else {
                                parts.join(", ")
                            }
                        })
                        .unwrap_or_else(|| "No Scan".to_string());

                    images.push(EcrImageInfo {
                        tag: tag.clone(),
                        digest: img.image_digest().unwrap_or("").to_string(),
                        size_mb,
                        pushed_at,
                        scan_status,
                    });
                }
            }
        }

        let created_at = repo
            .created_at()
            .map(|ts| {
                let secs = ts.secs() as i64;
                DateTime::from_timestamp(secs, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "-".to_string())
            })
            .unwrap_or_else(|| "-".to_string());

        let (encryption_type, kms_key) = repo
            .encryption_configuration()
            .map(|enc| {
                (
                    enc.encryption_type().as_str().to_string(),
                    enc.kms_key().map(|k| k.to_string()),
                )
            })
            .unwrap_or_else(|| ("AES256".to_string(), None));

        let tag_mutability = repo
            .image_tag_mutability()
            .map(|m| m.as_str())
            .unwrap_or("UNKNOWN")
            .to_string();

        Some(EcrDetail {
            name: repo.repository_name().unwrap_or("").to_string(),
            uri: repo.repository_uri().unwrap_or("").to_string(),
            tag_mutability,
            encryption_type,
            kms_key,
            created_at,
            image_count,
            images,
        })
    })
}
