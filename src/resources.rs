use aws_config::BehaviorVersion;
use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_rds::Client as RdsClient;
use aws_sdk_s3::Client as S3Client;

use crate::error::{AppError, Result};

/// Displays AWS EC2 instances, S3 buckets, and RDS clusters for the given region or the configured default.
///
/// The function builds an AWS SDK configuration scoped to `region` when provided (otherwise uses the default),
/// prints a header with the resolved region, invokes helpers to list EC2 instances, S3 buckets, and RDS instances,
/// then prints a closing banner.
///
/// # Examples
///
/// ```
/// # // Run these examples with an async runtime (e.g., Tokio)
/// #[tokio::main]
/// async fn main() {
///     // Show resources for the default configured region
///     let _ = show_resources(None).await;
/// }
/// ```
///
/// # Returns
///
/// `Ok(())` on success, or an `Err` value if any AWS SDK call or display step fails.
pub async fn show_resources(region: Option<String>) -> Result<()> {
    let config = if let Some(region) = &region {
        aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(region.clone()))
            .load()
            .await
    } else {
        aws_config::load_defaults(BehaviorVersion::latest()).await
    };

    let region_name = config
        .region()
        .map(|r| r.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!();
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!(
        "║              🌏 AWS Resources ({:^20})            ║",
        region_name
    );
    println!("╠══════════════════════════════════════════════════════════════════╣");

    // EC2 Instances
    show_ec2_instances(&config).await?;

    // S3 Buckets
    show_s3_buckets(&config).await?;

    // RDS Clusters
    show_rds_clusters(&config).await?;

    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    Ok(())
}

/// Render a table of EC2 instances using the provided AWS SDK configuration.
///
/// Displays up to 10 instances with columns for instance ID, instance type, state (with a status icon),
/// and the `Name` tag. When there are more than 10 instances, prints a summary line indicating how many more exist.
///
/// # Returns
///
/// `Ok(())` on success. AWS SDK errors are mapped to `AppError::AwsError`.
///
/// # Examples
///
/// ```no_run
/// # async fn doc_example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = aws_config::load_from_env().await;
/// show_ec2_instances(&config).await?;
/// # Ok(())
/// # }
/// ```
async fn show_ec2_instances(config: &aws_config::SdkConfig) -> Result<()> {
    let client = Ec2Client::new(config);

    let resp = client
        .describe_instances()
        .send()
        .await
        .map_err(|e| AppError::AwsError(e.to_string()))?;

    let mut instances: Vec<(String, String, String, String)> = Vec::new();

    for reservation in resp.reservations() {
        for instance in reservation.instances() {
            let id = instance.instance_id().unwrap_or("-").to_string();
            let instance_type = instance
                .instance_type()
                .map(|t| t.as_str().to_string())
                .unwrap_or_else(|| "-".to_string());
            let state = instance
                .state()
                .and_then(|s| s.name())
                .map(|n| n.as_str().to_string())
                .unwrap_or_else(|| "-".to_string());
            let name = instance
                .tags()
                .iter()
                .find(|tag| tag.key() == Some("Name"))
                .and_then(|tag| tag.value())
                .unwrap_or("-")
                .to_string();

            instances.push((id, instance_type, state, name));
        }
    }

    println!(
        "║  EC2 Instances ({})                                              ",
        instances.len()
    );
    println!("╠══════════════════════════════════════════════════════════════════╣");

    if instances.is_empty() {
        println!("║  (no instances)                                                  ║");
    } else {
        for (id, itype, state, name) in instances.iter().take(10) {
            let state_icon = match state.as_str() {
                "running" => "🟢",
                "stopped" => "🔴",
                "pending" => "🟡",
                _ => "⚪",
            };
            println!(
                "║  {} {:20} │ {:10} │ {:8} │ {:15} ║",
                state_icon,
                truncate(&id, 20),
                truncate(&itype, 10),
                truncate(&state, 8),
                truncate(&name, 15)
            );
        }
        if instances.len() > 10 {
            println!(
                "║  ... and {} more                                              ║",
                instances.len() - 10
            );
        }
    }

    Ok(())
}

/// Displays up to ten S3 buckets from the provided AWS SDK configuration in a formatted table.
///
/// Prints a header showing the total bucket count, then lists each bucket's name and creation date
/// truncated to fit the table. If more than ten buckets exist, a summary line indicates how many
/// additional buckets are present.
///
/// # Returns
///
/// `Ok(())` on success, or an `AppError` if the AWS SDK request fails.
///
/// # Examples
///
/// ```
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let config = aws_config::load_from_env().await;
/// // assume show_s3_buckets is in scope
/// show_s3_buckets(&config).await.unwrap();
/// # });
/// ```
async fn show_s3_buckets(config: &aws_config::SdkConfig) -> Result<()> {
    let client = S3Client::new(config);

    let resp = client
        .list_buckets()
        .send()
        .await
        .map_err(|e| AppError::AwsError(e.to_string()))?;
    let buckets: Vec<_> = resp.buckets().iter().collect();

    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!(
        "║  S3 Buckets ({})                                                 ",
        buckets.len()
    );
    println!("╠══════════════════════════════════════════════════════════════════╣");

    if buckets.is_empty() {
        println!("║  (no buckets)                                                    ║");
    } else {
        for bucket in buckets.iter().take(10) {
            let name = bucket.name().unwrap_or("-");
            let created = bucket
                .creation_date()
                .map(|d| {
                    d.fmt(aws_sdk_s3::primitives::DateTimeFormat::DateTime)
                        .unwrap_or_default()
                })
                .unwrap_or_else(|| "-".to_string());
            println!(
                "║  📦 {:40} │ {:20} ║",
                truncate(name, 40),
                truncate(&created, 20)
            );
        }
        if buckets.len() > 10 {
            println!(
                "║  ... and {} more                                              ║",
                buckets.len() - 10
            );
        }
    }

    Ok(())
}

/// Truncates or pads a string to a fixed width, using "..." to indicate truncation.
///
/// If `s` is longer than `max_len`, returns a shortened string that ends with `"..."`.
/// If `s` fits within `max_len`, returns `s` right-padded with spaces to exactly `max_len`.
/// When `max_len` is less than 3 and `s` is longer than `max_len`, the result will be `"..."`.
///
/// # Examples
///
/// ```
/// assert_eq!(truncate("hello world", 8), "hello...");
/// assert_eq!(truncate("hi", 5), "hi   "); // padded with spaces to length 5
/// assert_eq!(truncate("abcdef", 3), "..."); // max_len < 3 truncation case
/// ```
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        format!("{:width$}", s, width = max_len)
    }
}

/// Displays a summary table of RDS DB instances for the provided AWS SDK configuration.
///
/// Prints up to 10 DB instances showing a status icon, identifier, engine, instance class, and status, and prints a summary line if more instances exist. AWS SDK errors from the DescribeDBInstances call are converted to `AppError::AwsError` and returned.
///
/// # Parameters
///
/// - `config`: AWS SDK configuration used to create the RDS client.
///
/// # Returns
///
/// `Ok(())` on success; `Err(AppError::AwsError(_))` if the AWS DescribeDBInstances call fails.
///
/// # Examples
///
/// ```no_run
/// # use aws_config;
/// # use tokio;
/// # async fn try_example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = aws_config::load_from_env().await;
/// // `show_rds_clusters` is async and returns a Result<(), AppError>
/// show_rds_clusters(&config).await?;
/// # Ok(())
/// # }
/// ```
async fn show_rds_clusters(config: &aws_config::SdkConfig) -> Result<()> {
    let client = RdsClient::new(config);

    // Get DB Instances
    let resp = client
        .describe_db_instances()
        .send()
        .await
        .map_err(|e| AppError::AwsError(e.to_string()))?;
    let instances: Vec<_> = resp.db_instances().iter().collect();

    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!(
        "║  RDS Instances ({})                                              ",
        instances.len()
    );
    println!("╠══════════════════════════════════════════════════════════════════╣");

    if instances.is_empty() {
        println!("║  (no RDS instances)                                              ║");
    } else {
        for db in instances.iter().take(10) {
            let id = db.db_instance_identifier().unwrap_or("-");
            let engine = db.engine().unwrap_or("-");
            let class = db.db_instance_class().unwrap_or("-");
            let status = db.db_instance_status().unwrap_or("-");

            let status_icon = match status {
                "available" => "🟢",
                "stopped" => "🔴",
                "starting" | "stopping" | "modifying" => "🟡",
                _ => "⚪",
            };

            println!(
                "║  {} {:25} │ {:12} │ {:12} │ {:8} ║",
                status_icon,
                truncate(id, 25),
                truncate(engine, 12),
                truncate(class, 12),
                truncate(status, 8)
            );
        }
        if instances.len() > 10 {
            println!(
                "║  ... and {} more                                              ║",
                instances.len() - 10
            );
        }
    }

    Ok(())
}