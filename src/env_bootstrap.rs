use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwsEnvStatus {
    pub access_key_set: bool,
    pub secret_key_set: bool,
    pub session_token_set: bool,
    pub profile: String,
    pub region: String,
}

pub fn load_dotenv_file() -> Option<PathBuf> {
    dotenvy::dotenv().ok()
}

pub fn capture_aws_env_status() -> AwsEnvStatus {
    let access_key_set = env_var_is_set("AWS_ACCESS_KEY_ID");
    let secret_key_set = env_var_is_set("AWS_SECRET_ACCESS_KEY");
    let session_token_set = env_var_is_set("AWS_SESSION_TOKEN");

    let profile = std::env::var("AWS_PROFILE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("AWS_DEFAULT_PROFILE")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "default".to_string());

    let region = std::env::var("AWS_REGION")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("AWS_DEFAULT_REGION")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "-".to_string());

    AwsEnvStatus {
        access_key_set,
        secret_key_set,
        session_token_set,
        profile,
        region,
    }
}

fn env_var_is_set(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}
