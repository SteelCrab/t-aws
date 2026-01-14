use aws_config::BehaviorVersion;
use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_rds::Client as RdsClient;
use aws_sdk_s3::Client as S3Client;

pub async fn show_resources(region: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
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

    // VPC Resources
    show_vpc_resources(&config).await?;

    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    Ok(())
}

async fn show_ec2_instances(
    config: &aws_config::SdkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Ec2Client::new(config);

    let resp = client.describe_instances().send().await?;

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

async fn show_s3_buckets(config: &aws_config::SdkConfig) -> Result<(), Box<dyn std::error::Error>> {
    let client = S3Client::new(config);

    let resp = client.list_buckets().send().await?;
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

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        format!("{:width$}", s, width = max_len)
    }
}

async fn show_rds_clusters(
    config: &aws_config::SdkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = RdsClient::new(config);

    // Get DB Instances
    let resp = client.describe_db_instances().send().await?;
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

async fn show_vpc_resources(
    config: &aws_config::SdkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Ec2Client::new(config);

    // 1. VPCs
    let vpc_resp = client.describe_vpcs().send().await?;
    let vpcs = vpc_resp.vpcs();

    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!(
        "║  VPCs ({})                                                       ",
        vpcs.len()
    );
    println!("╠══════════════════════════════════════════════════════════════════╣");

    if vpcs.is_empty() {
        println!("║  (no VPCs)                                                       ║");
    } else {
        for vpc in vpcs.iter().take(5) {
            let id = vpc.vpc_id().unwrap_or("-");
            let cidr = vpc.cidr_block().unwrap_or("-");
            let state = vpc.state().map(|s| s.as_str()).unwrap_or("-");
            let name = vpc
                .tags()
                .iter()
                .find(|tag| tag.key() == Some("Name"))
                .and_then(|tag| tag.value())
                .unwrap_or("-");

            println!(
                "║  vpc  {:20} │ {:15} │ {:8} │ {:15} ║",
                truncate(id, 20),
                truncate(cidr, 15),
                truncate(state, 8),
                truncate(name, 15)
            );
        }
    }

    // 2. Subnets
    let subnet_resp = client.describe_subnets().send().await?;
    let subnets = subnet_resp.subnets();

    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!(
        "║  Subnets ({})                                                    ",
        subnets.len()
    );
    println!("╠══════════════════════════════════════════════════════════════════╣");

    if subnets.is_empty() {
        println!("║  (no Subnets)                                                    ║");
    } else {
        for subnet in subnets.iter().take(5) {
            let id = subnet.subnet_id().unwrap_or("-");
            let cidr = subnet.cidr_block().unwrap_or("-");
            let az = subnet.availability_zone().unwrap_or("-");
            let name = subnet
                .tags()
                .iter()
                .find(|tag| tag.key() == Some("Name"))
                .and_then(|tag| tag.value())
                .unwrap_or("-");

            println!(
                "║  sub  {:20} │ {:15} │ {:10} │ {:12} ║",
                truncate(id, 20),
                truncate(cidr, 15),
                truncate(az, 10),
                truncate(name, 12)
            );
        }
    }

    // 3. Security Groups
    let sg_resp = client.describe_security_groups().send().await?;
    let sgs = sg_resp.security_groups();

    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!(
        "║  Security Groups ({})                                            ",
        sgs.len()
    );
    println!("╠══════════════════════════════════════════════════════════════════╣");

    if sgs.is_empty() {
        println!("║  (no Security Groups)                                            ║");
    } else {
        for sg in sgs.iter().take(5) {
            let id = sg.group_id().unwrap_or("-");
            let name = sg.group_name().unwrap_or("-");
            let vpc_id = sg.vpc_id().unwrap_or("-");
            // let desc = sg.description().unwrap_or("-");

            println!(
                "║  sg   {:20} │ {:20} │ {:12} ║",
                truncate(id, 20),
                truncate(name, 20),
                truncate(vpc_id, 12)
            );
        }
    }

    Ok(())
}
