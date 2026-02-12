use serde::Deserialize;
use std::process::Command;
use std::sync::Mutex;

static REGION: Mutex<Option<String>> = Mutex::new(None);

pub fn set_region(region: &str) {
    if let Ok(mut r) = REGION.lock() {
        *r = Some(region.to_string());
    }
}

pub fn get_region_args() -> Vec<String> {
    if let Ok(r) = REGION.lock()
        && let Some(ref region) = *r
    {
        return vec!["--region".to_string(), region.clone()];
    }
    Vec::new()
}

#[derive(Debug, Clone)]
pub struct AwsResource {
    pub name: String,
    pub id: String,
    pub state: String,
    pub az: String,
    pub cidr: String,
}

impl AwsResource {
    pub fn display(&self) -> String {
        if self.name.is_empty() {
            self.id.clone()
        } else {
            format!("{} ({})", self.name, self.id)
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub key: String,
    pub value: String,
}

pub fn run_aws_cli(args: &[&str]) -> Option<String> {
    use std::io::Write;

    let region_args = get_region_args();
    let mut cmd = Command::new("aws");
    cmd.args(args);
    for arg in &region_args {
        cmd.arg(arg);
    }

    let cmd_str = format!("aws {} {}", args.join(" "), region_args.join(" "));
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/emd_debug.log")
    {
        let _ = writeln!(f, "[START] {}", cmd_str);
    }

    let output = cmd.output().ok()?;

    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/awsmd_debug.log")
    {
        let _ = writeln!(
            f,
            "[END] {} - success: {}, stdout_len: {}",
            cmd_str,
            output.status.success(),
            output.stdout.len()
        );
    }

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

pub fn check_aws_login() -> Result<String, String> {
    let output = Command::new("aws")
        .args(["sts", "get-caller-identity", "--output", "json"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let json = String::from_utf8_lossy(&o.stdout);
            let account = extract_json_value(&json, "Account").unwrap_or_default();
            let arn = extract_json_value(&json, "Arn").unwrap_or_default();
            Ok(format!("{} ({})", account, arn))
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            Err(format!(
                "AWS 로그인 필요: {}",
                err.lines().next().unwrap_or("")
            ))
        }
        Err(e) => Err(format!("AWS CLI 실행 실패: {}", e)),
    }
}

pub fn extract_json_value(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\": \"", key);
    if let Some(start) = json.find(&pattern) {
        let offset = start + pattern.len();
        if let Some(end) = json[offset..].find('"') {
            return Some(json[offset..offset + end].to_string());
        }
    }
    None
}

pub fn parse_name_tag(tags_json: &str) -> String {
    if let Some(start) = tags_json.find("\"Key\": \"Name\"")
        && let Some(value_start) = tags_json[start..].find("\"Value\": \"")
    {
        let offset = start + value_start + 10;
        if let Some(end) = tags_json[offset..].find('"') {
            return tags_json[offset..offset + end].to_string();
        }
    }
    if let Some(start) = tags_json.find("\"Value\": \"") {
        let offset = start + 10;
        if let Some(end) = tags_json[offset..].find('"') {
            let value = &tags_json[offset..offset + end];
            if tags_json[offset + end..].contains("\"Key\": \"Name\"") {
                return value.to_string();
            }
        }
    }
    String::new()
}

pub fn extract_tags(json: &str) -> Vec<(String, String)> {
    let mut tags = Vec::new();
    let mut search_start = 0;

    while let Some(key_pos) = json[search_start..].find("\"Key\": \"") {
        let key_start = search_start + key_pos + 8;
        if let Some(key_end) = json[key_start..].find('"') {
            let key = json[key_start..key_start + key_end].to_string();

            if let Some(val_pos) = json[key_start..].find("\"Value\": \"") {
                let val_start = key_start + val_pos + 10;
                if let Some(val_end) = json[val_start..].find('"') {
                    let value = json[val_start..val_start + val_end].to_string();
                    if !tags.iter().any(|(k, _)| k == &key) {
                        tags.push((key, value));
                    }
                }
            }
            search_start = key_start + key_end;
        } else {
            break;
        }
    }
    tags
}

pub fn parse_resources_from_json(json: &str, prefix: &str) -> Vec<AwsResource> {
    let mut resources = Vec::new();

    let mut search_start = 0;
    while let Some(pos) = json[search_start..].find(prefix) {
        let start = search_start + pos;
        if let Some(end) = json[start..].find('"') {
            let id = &json[start..start + end];
            if id.starts_with(prefix) && !id.contains(' ') {
                let section_end = json[start..]
                    .find(']')
                    .map(|p| start + p)
                    .unwrap_or(json.len());
                let tag_start = start;
                let tag_end = section_end;
                let tags_json = &json[tag_start..tag_end];
                let name = parse_name_tag(tags_json);

                resources.push(AwsResource {
                    name,
                    id: id.to_string(),
                    state: String::new(),
                    az: String::new(),
                    cidr: String::new(),
                });
            }
            search_start = start + end;
        } else {
            break;
        }
    }
    resources
}

use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create Tokio runtime"))
}

/// Get AWS SDK config with profile-based credentials and region
pub async fn get_sdk_config() -> aws_config::SdkConfig {
    let profile_name = std::env::var("AWS_PROFILE").unwrap_or_else(|_| "default".to_string());

    let mut config_loader =
        aws_config::defaults(aws_config::BehaviorVersion::latest()).profile_name(&profile_name);

    // Get region from REGION mutex if set
    if let Ok(r) = REGION.lock()
        && let Some(ref region_str) = *r
    {
        config_loader = config_loader.region(aws_config::Region::new(region_str.clone()));
    }

    config_loader.load().await
}
