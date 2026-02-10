use crate::aws_cli::common::{extract_json_value, run_aws_cli};
use serde::Deserialize;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IamRoleDetail {
    pub name: String,
    pub arn: String,
    pub assume_role_policy: String,
    pub attached_policies: Vec<AttachedPolicy>,
    pub inline_policies: Vec<InlinePolicy>,
}

#[derive(Debug, Clone)]
pub struct AttachedPolicy {
    pub name: String,
    pub arn: String,
}

#[derive(Debug, Clone)]
pub struct InlinePolicy {
    pub name: String,
    pub document: String,
}

pub fn get_iam_role_detail(role_name: &str) -> Option<IamRoleDetail> {
    // 역할 기본 정보
    let output = run_aws_cli(&[
        "iam",
        "get-role",
        "--role-name",
        role_name,
        "--output",
        "json",
    ])?;

    let name = extract_json_value(&output, "RoleName").unwrap_or_default();
    let arn = extract_json_value(&output, "Arn").unwrap_or_default();

    // AssumeRolePolicyDocument 파싱
    let assume_role_policy = extract_assume_role_policy(&output);

    // 연결된 정책
    let attached_policies = get_attached_policies(role_name);

    // 인라인 정책
    let inline_policies = get_inline_policies(role_name);

    Some(IamRoleDetail {
        name,
        arn,
        assume_role_policy,
        attached_policies,
        inline_policies,
    })
}

fn extract_assume_role_policy(json: &str) -> String {
    if let Some(start) = json.find("\"AssumeRolePolicyDocument\":") {
        let after_key = &json[start + 27..];
        if let Some(obj_start) = after_key.find('{') {
            let obj_json = &after_key[obj_start..];
            let mut depth = 0;
            let mut end_idx = 0;
            for (i, c) in obj_json.chars().enumerate() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end_idx > 0 {
                let policy_json = &obj_json[..end_idx];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(policy_json)
                    && let Ok(pretty) = serde_json::to_string_pretty(&parsed)
                {
                    return pretty;
                }
                return policy_json.to_string();
            }
        }
    }
    String::new()
}

fn get_attached_policies(role_name: &str) -> Vec<AttachedPolicy> {
    let output = match run_aws_cli(&[
        "iam",
        "list-attached-role-policies",
        "--role-name",
        role_name,
        "--output",
        "json",
    ]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    #[derive(Deserialize)]
    struct Response {
        #[serde(rename = "AttachedPolicies")]
        attached_policies: Vec<Policy>,
    }

    #[derive(Deserialize)]
    struct Policy {
        #[serde(rename = "PolicyName")]
        policy_name: String,
        #[serde(rename = "PolicyArn")]
        policy_arn: String,
    }

    if let Ok(response) = serde_json::from_str::<Response>(&output) {
        return response
            .attached_policies
            .into_iter()
            .map(|p| AttachedPolicy {
                name: p.policy_name,
                arn: p.policy_arn,
            })
            .collect();
    }

    Vec::new()
}

fn get_inline_policies(role_name: &str) -> Vec<InlinePolicy> {
    let output = match run_aws_cli(&[
        "iam",
        "list-role-policies",
        "--role-name",
        role_name,
        "--output",
        "json",
    ]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    #[derive(Deserialize)]
    struct Response {
        #[serde(rename = "PolicyNames")]
        policy_names: Vec<String>,
    }

    let policy_names = match serde_json::from_str::<Response>(&output) {
        Ok(r) => r.policy_names,
        Err(_) => return Vec::new(),
    };

    let mut policies = Vec::new();

    for policy_name in policy_names {
        if let Some(doc) = get_inline_policy_document(role_name, &policy_name) {
            policies.push(InlinePolicy {
                name: policy_name,
                document: doc,
            });
        }
    }

    policies
}

fn get_inline_policy_document(role_name: &str, policy_name: &str) -> Option<String> {
    let output = run_aws_cli(&[
        "iam",
        "get-role-policy",
        "--role-name",
        role_name,
        "--policy-name",
        policy_name,
        "--output",
        "json",
    ])?;

    if let Some(start) = output.find("\"PolicyDocument\":") {
        let after_key = &output[start + 17..];
        if let Some(obj_start) = after_key.find('{') {
            let obj_json = &after_key[obj_start..];
            let mut depth = 0;
            let mut end_idx = 0;
            for (i, c) in obj_json.chars().enumerate() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end_idx > 0 {
                let policy_json = &obj_json[..end_idx];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(policy_json)
                    && let Ok(pretty) = serde_json::to_string_pretty(&parsed)
                {
                    return Some(pretty);
                }
                return Some(policy_json.to_string());
            }
        }
    }

    None
}
