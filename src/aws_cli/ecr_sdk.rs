use crate::aws_cli::common::{AwsResource, run_aws_cli};
use crate::aws_cli::ecr::{
    EcrDetail, EcrImagesResponse, EcrRepositoriesResponse, format_created_at, mutability_label,
};

pub fn list_ecr_repositories() -> Vec<AwsResource> {
    let output = match run_aws_cli(&["ecr", "describe-repositories", "--output", "json"]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    parse_repository_list_output(&output).unwrap_or_default()
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

    let images_output = run_aws_cli(&[
        "ecr",
        "describe-images",
        "--repository-name",
        repo_name,
        "--output",
        "json",
    ]);

    build_ecr_detail_from_outputs(&output, images_output.as_deref())
}

fn parse_repository_list_output(output: &str) -> Option<Vec<AwsResource>> {
    let response: EcrRepositoriesResponse = serde_json::from_str(output).ok()?;

    Some(
        response
            .repositories
            .into_iter()
            .map(|repo| {
                let mutability = mutability_label(&repo.image_tag_mutability);

                AwsResource {
                    name: format!("{} ({})", repo.repository_name, mutability),
                    id: repo.repository_name,
                    state: repo.image_tag_mutability,
                    az: String::new(),
                    cidr: repo.repository_uri,
                }
            })
            .collect(),
    )
}

fn build_ecr_detail_from_outputs(
    repo_output: &str,
    image_output: Option<&str>,
) -> Option<EcrDetail> {
    let response: EcrRepositoriesResponse = serde_json::from_str(repo_output).ok()?;
    let repo = response.repositories.first()?;

    let image_count = image_output
        .and_then(|o| serde_json::from_str::<EcrImagesResponse>(o).ok())
        .map(|r| r.image_details.len() as i32)
        .unwrap_or(0);

    let created_at = format_created_at(repo.created_at);

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

#[cfg(test)]
mod tests {
    use super::{build_ecr_detail_from_outputs, parse_repository_list_output};

    #[test]
    fn parse_repository_list_output_maps_mutability_and_name() {
        let payload = r#"
            {
              "repositories": [
                {
                  "repositoryName": "repo-a",
                  "repositoryUri": "123456789012.dkr.ecr.ap-northeast-2.amazonaws.com/repo-a",
                  "imageTagMutability": "IMMUTABLE",
                  "createdAt": 1700000000.0
                }
              ]
            }
        "#;

        let resources = parse_repository_list_output(payload).expect("parse list");
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].id, "repo-a");
        assert!(resources[0].name.contains("Immutable"));
    }

    #[test]
    fn build_ecr_detail_from_outputs_handles_defaults() {
        let repo_payload = r#"
            {
              "repositories": [
                {
                  "repositoryName": "repo-b",
                  "repositoryUri": "123456789012.dkr.ecr.ap-northeast-2.amazonaws.com/repo-b",
                  "imageTagMutability": "MUTABLE",
                  "createdAt": 1700000000.0
                }
              ]
            }
        "#;
        let image_payload = r#"
            {
              "imageDetails": [
                {"imageDigest": "sha256:1"},
                {"imageDigest": "sha256:2"}
              ]
            }
        "#;

        let detail =
            build_ecr_detail_from_outputs(repo_payload, Some(image_payload)).expect("detail");
        assert_eq!(detail.name, "repo-b");
        assert_eq!(detail.image_count, 2);
        assert_eq!(detail.encryption_type, "AES256");
    }
}
