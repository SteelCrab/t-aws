use crate::aws_cli::common::{
    AwsResource, Tag, extract_json_value, extract_tags, parse_name_tag, parse_resources_from_json,
};
use crate::i18n::{I18n, Language};
use serde::Deserialize;
use std::collections::HashMap;

#[cfg(not(test))]
mod cli_adapter {
    pub(super) fn run(args: &[&str]) -> Option<String> {
        crate::aws_cli::common::run_aws_cli(args)
    }
}

#[cfg(test)]
mod cli_adapter {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static RESPONSES: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();

    fn key(args: &[&str]) -> String {
        args.join("\x1f")
    }

    pub(super) fn run(args: &[&str]) -> Option<String> {
        RESPONSES
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .ok()
            .and_then(|map| map.get(&key(args)).cloned().flatten())
    }

    pub(super) fn set(args: &[&str], output: Option<&str>) {
        if let Ok(mut map) = RESPONSES.get_or_init(|| Mutex::new(HashMap::new())).lock() {
            map.insert(key(args), output.map(str::to_string));
        }
    }

    pub(super) fn clear() {
        if let Ok(mut map) = RESPONSES.get_or_init(|| Mutex::new(HashMap::new())).lock() {
            map.clear();
        }
    }
}

#[cfg(not(test))]
mod subnet_adapter {
    pub(super) fn get(subnet_id: &str) -> String {
        crate::aws_cli::ec2::get_subnet_name(subnet_id)
    }
}

#[cfg(test)]
mod subnet_adapter {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static RESPONSES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

    pub(super) fn get(subnet_id: &str) -> String {
        RESPONSES
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .ok()
            .and_then(|map| map.get(subnet_id).cloned())
            .unwrap_or_else(|| subnet_id.to_string())
    }

    pub(super) fn set(subnet_id: &str, subnet_name: &str) {
        if let Ok(mut map) = RESPONSES.get_or_init(|| Mutex::new(HashMap::new())).lock() {
            map.insert(subnet_id.to_string(), subnet_name.to_string());
        }
    }

    pub(super) fn clear() {
        if let Ok(mut map) = RESPONSES.get_or_init(|| Mutex::new(HashMap::new())).lock() {
            map.clear();
        }
    }
}

// Serde structures
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SubnetResponse {
    subnets: Vec<Subnet>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Subnet {
    subnet_id: String,
    vpc_id: String,
    cidr_block: String,
    availability_zone: String,
    state: String,
    #[serde(default)]
    map_public_ip_on_launch: bool,
    #[serde(default)]
    available_ip_address_count: i32,
    #[serde(default)]
    tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NatGatewayResponse {
    nat_gateways: Vec<NatGateway>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NatGateway {
    nat_gateway_id: String,
    #[serde(default)]
    subnet_id: String,
    state: String,
    #[serde(default)]
    connectivity_type: String,
    #[serde(default)]
    availability_mode: String,
    #[serde(default)]
    auto_scaling_ips: String,
    #[serde(default)]
    auto_provision_zones: String,
    #[serde(default)]
    nat_gateway_addresses: Vec<NatGatewayAddress>,
    #[serde(default)]
    tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NatGatewayAddress {
    #[serde(default)]
    allocation_id: String,
    #[serde(default)]
    public_ip: String,
    #[allow(dead_code)]
    #[serde(default)]
    private_ip: String,
}

// Detail structures
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct NatDetail {
    pub name: String,
    pub id: String,
    pub state: String,
    pub connectivity_type: String,
    pub availability_mode: String,
    pub auto_scaling_ips: String,
    pub auto_provision_zones: String,
    pub public_ip: String,
    pub allocation_id: String,
    pub subnet_id: String,
    pub tags: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteTableDetail {
    pub name: String,
    #[allow(dead_code)]
    pub id: String,
    pub routes: Vec<String>,
    pub associations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EipDetail {
    pub name: String,
    pub public_ip: String,
    pub instance_id: String,
    pub private_ip: String,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct NetworkDetail {
    pub name: String,
    pub id: String,
    pub cidr: String,
    pub state: String,
    pub subnets: Vec<AwsResource>,
    pub igws: Vec<AwsResource>,
    pub nats: Vec<NatDetail>,
    pub route_tables: Vec<RouteTableDetail>,
    pub eips: Vec<EipDetail>,
    pub dns_support: bool,
    pub dns_hostnames: bool,
    pub tags: Vec<(String, String)>,
}

impl NetworkDetail {
    pub fn to_markdown(&self, lang: Language) -> String {
        let i18n = I18n::new(lang);
        let vpc_display = if self.name.is_empty() || self.name == self.id {
            format!("NULL - {}", self.id)
        } else {
            format!("{} - {}", self.name, self.id)
        };
        let mut lines = vec![format!("## Network ({})\n", vpc_display)];

        lines.push(format!("| {} | {} |", i18n.item(), i18n.value()));
        lines.push("|:---|:---|".to_string());
        lines.push(format!("| {} | {} |", i18n.md_name(), vpc_display));
        lines.push(format!("| CIDR | {} |", self.cidr));
        lines.push(format!(
            "| {} | {} |",
            i18n.md_dns_support(),
            self.dns_support
        ));
        lines.push(format!(
            "| {} | {} |",
            i18n.md_dns_hostnames(),
            self.dns_hostnames
        ));

        for (key, value) in &self.tags {
            if key != "Name" {
                lines.push(format!("| {}-{} | {} |", i18n.tag(), key, value));
            }
        }

        if !self.subnets.is_empty() {
            lines.push(format!("\n### {}", i18n.md_subnets()));
            lines.push(format!(
                "| {} | CIDR | AZ | {} |",
                i18n.md_name(),
                i18n.md_state()
            ));
            lines.push("|:---|:---|:---|:---|".to_string());
            for subnet in &self.subnets {
                lines.push(format!(
                    "| {} | {} | {} | {} |",
                    subnet.name, subnet.cidr, subnet.az, subnet.state
                ));
            }
        }

        if !self.igws.is_empty() {
            lines.push(format!("\n### {}", i18n.md_internet_gateway()));
            lines.push(format!(
                "| {} | {} |",
                i18n.md_name(),
                i18n.md_attached_vpc()
            ));
            lines.push("|:---|:---|".to_string());
            for igw in &self.igws {
                let igw_display = if igw.name.is_empty() || igw.name == igw.id {
                    format!("NULL - {}", igw.id)
                } else {
                    format!("{} - {}", igw.name, igw.id)
                };
                lines.push(format!("| {} | {} |", igw_display, vpc_display));
            }
        }

        if !self.nats.is_empty() {
            lines.push(format!("\n### {}", i18n.md_nat_gateway()));
            for nat in &self.nats {
                let nat_display = if nat.name.is_empty() || nat.name == nat.id {
                    format!("NULL - {}", nat.id)
                } else {
                    format!("{} - {}", nat.name, nat.id)
                };
                lines.push(format!("\n#### {}", nat_display));
                lines.push(format!("| {} | {} |", i18n.item(), i18n.value()));
                lines.push("|:---|:---|".to_string());
                lines.push(format!("| {} | {} |", i18n.md_name(), nat_display));

                let is_regional = nat.availability_mode == "regional";
                let availability_mode = if nat.availability_mode.is_empty() {
                    i18n.md_zonal().to_string()
                } else if is_regional {
                    i18n.md_regional().to_string()
                } else {
                    i18n.md_zonal().to_string()
                };
                lines.push(format!(
                    "| {} | {} |",
                    i18n.md_availability_mode(),
                    availability_mode
                ));

                if is_regional {
                    let auto_scaling = if nat.auto_scaling_ips == "enabled" {
                        i18n.md_enabled()
                    } else {
                        i18n.md_disabled()
                    };
                    lines.push(format!(
                        "| {} | {} |",
                        i18n.md_ip_auto_scaling(),
                        auto_scaling
                    ));

                    let auto_provision = if nat.auto_provision_zones == "enabled" {
                        i18n.md_enabled()
                    } else {
                        i18n.md_disabled()
                    };
                    lines.push(format!(
                        "| {} | {} |",
                        i18n.md_zone_auto_provisioning(),
                        auto_provision
                    ));
                } else {
                    let subnet_display = self
                        .subnets
                        .iter()
                        .find(|s| s.id == nat.subnet_id)
                        .map(|s| {
                            if s.name.is_empty() {
                                format!("NULL - {}", s.id)
                            } else {
                                format!("{} - {}", s.name, s.id)
                            }
                        })
                        .unwrap_or_else(|| {
                            if nat.subnet_id.is_empty() {
                                "-".to_string()
                            } else {
                                format!("NULL - {}", nat.subnet_id)
                            }
                        });
                    lines.push(format!("| {} | {} |", i18n.md_subnet(), subnet_display));
                }

                let conn_type =
                    if nat.connectivity_type.is_empty() || nat.connectivity_type == "public" {
                        i18n.md_public()
                    } else {
                        i18n.md_private()
                    };
                lines.push(format!(
                    "| {} | {} |",
                    i18n.md_connectivity_type(),
                    conn_type
                ));

                if !nat.allocation_id.is_empty() {
                    lines.push(format!(
                        "| {} | `{}` |",
                        i18n.md_elastic_ip_allocation_id(),
                        nat.allocation_id
                    ));
                }

                for (key, value) in &nat.tags {
                    lines.push(format!("| {}-{} | {} |", i18n.tag(), key, value));
                }
            }
        }

        if !self.route_tables.is_empty() {
            lines.push(format!("\n### {}", i18n.md_route_tables()));
            for rt in &self.route_tables {
                let display_name = if rt.name.is_empty() {
                    format!("NULL - {}", rt.id)
                } else {
                    format!("{} - {}", rt.name, rt.id)
                };
                lines.push(format!("\n#### {}", display_name));

                if !rt.routes.is_empty() {
                    lines.push(format!(
                        "| {} | {} | {} |",
                        i18n.md_destination(),
                        i18n.md_target(),
                        i18n.md_state()
                    ));
                    lines.push("|:---|:---|:---|".to_string());
                    for route in &rt.routes {
                        let parts: Vec<&str> = route.split('|').collect();
                        if parts.len() >= 3 {
                            lines.push(format!("| {} | {} | {} |", parts[0], parts[1], parts[2]));
                        }
                    }
                }

                if !rt.associations.is_empty() {
                    lines.push(format!("\n**{}**", i18n.md_associated_subnets()));
                    lines.push(format!("| {} |", i18n.md_subnet()));
                    lines.push("|:---|".to_string());
                    for assoc in &rt.associations {
                        lines.push(format!("| {} |", assoc));
                    }
                }
            }
        }

        if !self.eips.is_empty() {
            lines.push("\n### Elastic IPs".to_string());
            lines.push(format!(
                "| {} | Public IP | {} |",
                i18n.md_name(),
                i18n.md_association()
            ));
            lines.push("|:---|:---|:---|".to_string());
            for eip in &self.eips {
                let assoc = if !eip.instance_id.is_empty() {
                    format!("Instance: {}", eip.instance_id)
                } else if !eip.private_ip.is_empty() {
                    format!("Private IP: {}", eip.private_ip)
                } else {
                    "-".to_string()
                };
                lines.push(format!("| {} | {} | {} |", eip.name, eip.public_ip, assoc));
            }
        }

        // Network Diagram
        lines.push(format!("\n### {}", i18n.md_network_diagram()));
        lines.push(self.generate_mermaid());

        lines.join("\n") + "\n"
    }

    fn generate_mermaid(&self) -> String {
        let to_node_id = |name: &str| -> String {
            name.to_lowercase()
                .replace("-", "_")
                .replace(" ", "_")
                .replace("(", "")
                .replace(")", "")
        };

        let mut diagram = String::from("\n```mermaid\ngraph TD\n");

        diagram.push_str("    Internet((\"☁️ Internet\"))\n");
        diagram.push_str("    style Internet fill:#fff9c4,stroke:#f57f17\n\n");

        diagram.push_str(&format!(
            "    subgraph VPC[\"{} ({})\"]\n",
            self.name, self.cidr
        ));
        diagram.push_str("    style VPC fill:#e1f5fe,stroke:#01579b\n");

        for igw in &self.igws {
            let igw_node_id = to_node_id(&igw.name);
            diagram.push_str(&format!("    {}[\"🌐 {}\"]\n", igw_node_id, igw.name));
            diagram.push_str(&format!(
                "    style {} fill:#fff3e0,stroke:#e65100\n",
                igw_node_id
            ));
        }

        let mut nat_by_subnet: HashMap<String, &NatDetail> = HashMap::new();
        for nat in &self.nats {
            nat_by_subnet.insert(nat.subnet_id.clone(), nat);
        }

        let mut id_to_name: HashMap<String, String> = HashMap::new();
        for igw in &self.igws {
            id_to_name.insert(igw.id.clone(), igw.name.clone());
        }
        for nat in &self.nats {
            id_to_name.insert(nat.id.clone(), nat.name.clone());
        }
        for subnet in &self.subnets {
            id_to_name.insert(subnet.id.clone(), subnet.name.clone());
        }

        let mut subnets_by_az: HashMap<String, Vec<&AwsResource>> = HashMap::new();
        for s in &self.subnets {
            let az = if s.az.is_empty() {
                "unknown".to_string()
            } else {
                s.az.clone()
            };
            subnets_by_az.entry(az).or_default().push(s);
        }

        for (az, subnets) in &subnets_by_az {
            let az_clean = az.replace("-", "_");
            diagram.push_str(&format!("\n        subgraph {}[\"📍 {}\"]\n", az_clean, az));
            diagram.push_str(&format!(
                "        style {} fill:#f3e5f5,stroke:#4a148c,stroke-dasharray: 5 5\n",
                az_clean
            ));

            for subnet in subnets {
                let subnet_node_id = to_node_id(&subnet.name);
                diagram.push_str(&format!(
                    "            {}[\"{}\\n{}\"]\n",
                    subnet_node_id, subnet.name, subnet.cidr
                ));
                diagram.push_str(&format!(
                    "            style {} fill:#e8f5e9,stroke:#1b5e20\n",
                    subnet_node_id
                ));

                if let Some(nat) = nat_by_subnet.get(&subnet.id) {
                    let nat_node_id = to_node_id(&nat.name);
                    diagram.push_str(&format!(
                        "            {}[\"🔀 {}\"]\n",
                        nat_node_id, nat.name
                    ));
                    diagram.push_str(&format!(
                        "            style {} fill:#ffecb3,stroke:#ff6f00\n",
                        nat_node_id
                    ));
                }
            }
            diagram.push_str("        end\n");
        }

        diagram.push('\n');

        if let Some(igw) = self.igws.first() {
            let igw_node_id = to_node_id(&igw.name);
            diagram.push_str(&format!("    Internet <--> {}\n", igw_node_id));
        }

        for rt in &self.route_tables {
            let mut target_id: Option<String> = None;
            for route in &rt.routes {
                // Format: "dest|target|state"
                let parts: Vec<&str> = route.split('|').collect();
                if parts.len() >= 2 && parts[0] == "0.0.0.0/0" {
                    let target = parts[1];
                    if target.starts_with("igw-") || target.starts_with("nat-") {
                        target_id = Some(target.to_string());
                        break;
                    }
                }
            }

            for assoc in &rt.associations {
                if let Some(start) = assoc.find("(subnet-")
                    && let Some(end) = assoc[start + 1..].find(')')
                {
                    let subnet_id = &assoc[start + 1..start + 1 + end];

                    if let Some(subnet_name) = id_to_name.get(subnet_id) {
                        let subnet_node_id = to_node_id(subnet_name);

                        if let Some(ref target_aws_id) = target_id
                            && let Some(target_name) = id_to_name.get(target_aws_id)
                        {
                            let target_node_id = to_node_id(target_name);

                            if target_aws_id.starts_with("nat-") {
                                diagram.push_str(&format!(
                                    "    {} -.->|Private| {}\n",
                                    subnet_node_id, target_node_id
                                ));
                            } else if target_aws_id.starts_with("igw-") {
                                diagram.push_str(&format!(
                                    "    {} <-->|Public| {}\n",
                                    target_node_id, subnet_node_id
                                ));
                            }
                        }
                    }
                }
            }
        }

        for nat in &self.nats {
            if let Some(igw) = self.igws.first() {
                let nat_node_id = to_node_id(&nat.name);
                let igw_node_id = to_node_id(&igw.name);
                diagram.push_str(&format!("    {} ==> {}\n", nat_node_id, igw_node_id));
            }
        }

        diagram.push_str("    end\n```\n");
        diagram
    }
}

// Public functions
pub fn list_vpcs() -> Vec<AwsResource> {
    let command = [
        "ec2",
        "describe-vpcs",
        "--query",
        "Vpcs[*].[VpcId,Tags]",
        "--output",
        "json",
    ];
    let output = match cli_adapter::run(&command) {
        Some(o) => o,
        None => {
            tracing::warn!(command = %command.join(" "), "list_vpcs: no output from aws adapter");
            return Vec::new();
        }
    };
    tracing::debug!(
        command = %command.join(" "),
        bytes = output.len(),
        "list_vpcs: aws adapter response"
    );

    parse_resources_from_json(&output, "vpc-")
}

pub fn list_subnets(vpc_id: &str) -> Vec<AwsResource> {
    let command = ["ec2", "describe-subnets", "--output", "json"];
    let output = match cli_adapter::run(&command) {
        Some(o) => o,
        None => {
            tracing::warn!(
                vpc_id,
                command = %command.join(" "),
                "list_subnets: no output from aws adapter"
            );
            return Vec::new();
        }
    };
    tracing::debug!(
        vpc_id,
        command = %command.join(" "),
        bytes = output.len(),
        "list_subnets: aws adapter response"
    );

    parse_subnets_output(&output, vpc_id)
}

fn parse_subnets_output(output: &str, vpc_id: &str) -> Vec<AwsResource> {
    let response: SubnetResponse = match serde_json::from_str(output) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    response
        .subnets
        .into_iter()
        .filter(|s| s.vpc_id == vpc_id)
        .map(|s| {
            let name = s
                .tags
                .iter()
                .find(|t| t.key == "Name")
                .map(|t| t.value.clone())
                .unwrap_or_default();

            AwsResource {
                name,
                id: s.subnet_id,
                state: s.state,
                az: s.availability_zone,
                cidr: s.cidr_block,
            }
        })
        .collect()
}

pub fn list_internet_gateways(vpc_id: &str) -> Vec<AwsResource> {
    let filter = format!("Name=attachment.vpc-id,Values={}", vpc_id);
    let command = [
        "ec2",
        "describe-internet-gateways",
        "--filters",
        &filter,
        "--query",
        "InternetGateways[*].[InternetGatewayId,Tags,Attachments]",
        "--output",
        "json",
    ];
    let output = match cli_adapter::run(&command) {
        Some(o) => o,
        None => {
            tracing::warn!(
                vpc_id,
                command = %command.join(" "),
                filter = %filter,
                "list_internet_gateways: no output from aws adapter"
            );
            return Vec::new();
        }
    };
    tracing::debug!(
        vpc_id,
        command = %command.join(" "),
        bytes = output.len(),
        "list_internet_gateways: aws adapter response"
    );

    parse_internet_gateways(&output)
}

fn parse_internet_gateways(json: &str) -> Vec<AwsResource> {
    let mut resources = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = json[search_start..].find("igw-") {
        let start = search_start + pos;
        if let Some(end) = json[start..].find('"') {
            let id = &json[start..start + end];
            if !id.starts_with("igw-") {
                search_start = start + end;
                continue;
            }

            let tag_start_offset = json[start..].find('[').unwrap_or_default();
            let tag_start = start + tag_start_offset;
            let tag_end_offset = json[tag_start..].find(']').unwrap_or_default();
            let tag_end = tag_start + tag_end_offset + 1;
            let tags_json = &json[tag_start..tag_end];
            let name = parse_name_tag(tags_json);

            let att_start_offset = json[tag_end..].find('[').unwrap_or_default();
            let att_start = tag_end + att_start_offset;
            let att_end_offset = json[att_start..].find(']').unwrap_or_default();
            let att_end = att_start + att_end_offset + 1;
            let att_json = &json[att_start..att_end];

            let state = if att_json.contains("\"State\": \"available\"")
                || att_json.contains("\"State\": \"attached\"")
                || att_json.contains("vpc-")
            {
                "attached".to_string()
            } else {
                "detached".to_string()
            };

            resources.push(AwsResource {
                name,
                id: id.to_string(),
                state,
                az: String::new(),
                cidr: String::new(),
            });

            search_start = att_end;
        } else {
            break;
        }
    }
    resources
}

pub fn list_nat_gateways(vpc_id: &str) -> Vec<NatDetail> {
    let filter = format!("Name=vpc-id,Values={}", vpc_id);
    let command = [
        "ec2",
        "describe-nat-gateways",
        "--filter",
        &filter,
        "--output",
        "json",
    ];
    let output = match cli_adapter::run(&command) {
        Some(o) => o,
        None => {
            tracing::warn!(
                vpc_id,
                command = %command.join(" "),
                filter = %filter,
                "list_nat_gateways: no output from aws adapter"
            );
            return Vec::new();
        }
    };
    tracing::debug!(
        vpc_id,
        command = %command.join(" "),
        bytes = output.len(),
        "list_nat_gateways: aws adapter response"
    );

    parse_nat_gateways_output(&output)
}

fn parse_nat_gateways_output(output: &str) -> Vec<NatDetail> {
    let response: NatGatewayResponse = match serde_json::from_str(output) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    response
        .nat_gateways
        .into_iter()
        .filter(|nat| nat.state != "deleted")
        .map(|nat| {
            let name = nat
                .tags
                .iter()
                .find(|t| t.key == "Name")
                .map(|t| t.value.clone())
                .unwrap_or_default();

            let (public_ip, allocation_id) = nat
                .nat_gateway_addresses
                .first()
                .map(|addr| (addr.public_ip.clone(), addr.allocation_id.clone()))
                .unwrap_or_default();

            let tags: Vec<(String, String)> = nat
                .tags
                .iter()
                .filter(|t| t.key != "Name")
                .map(|t| (t.key.clone(), t.value.clone()))
                .collect();

            NatDetail {
                name,
                id: nat.nat_gateway_id,
                state: nat.state,
                connectivity_type: nat.connectivity_type,
                availability_mode: nat.availability_mode,
                auto_scaling_ips: nat.auto_scaling_ips,
                auto_provision_zones: nat.auto_provision_zones,
                public_ip,
                allocation_id,
                subnet_id: nat.subnet_id,
                tags,
            }
        })
        .collect()
}

pub fn list_route_tables(vpc_id: &str) -> Vec<RouteTableDetail> {
    let filter = format!("Name=vpc-id,Values={}", vpc_id);
    let command = [
        "ec2",
        "describe-route-tables",
        "--filters",
        &filter,
        "--query",
        "RouteTables[*].[RouteTableId,Tags,Routes,Associations]",
        "--output",
        "json",
    ];
    let output = match cli_adapter::run(&command) {
        Some(o) => o,
        None => {
            tracing::warn!(
                vpc_id,
                command = %command.join(" "),
                filter = %filter,
                "list_route_tables: no output from aws adapter"
            );
            return Vec::new();
        }
    };
    tracing::debug!(
        vpc_id,
        command = %command.join(" "),
        bytes = output.len(),
        "list_route_tables: aws adapter response"
    );

    parse_route_tables(&output)
}

pub fn parse_route_tables(json: &str) -> Vec<RouteTableDetail> {
    let mut details = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = json[search_start..].find("rtb-") {
        let start = search_start + pos;
        if let Some(end) = json[start..].find('"') {
            let id = &json[start..start + end];

            if !id.starts_with("rtb-") {
                search_start = start + end;
                continue;
            }

            let tag_start_offset = json[start..].find('[').unwrap_or_default();
            let tag_start = start + tag_start_offset;
            let tag_end_offset = json[tag_start..].find(']').unwrap_or_default();
            let tag_end = tag_start + tag_end_offset + 1;
            let tags_json = &json[tag_start..tag_end];
            let name = parse_name_tag(tags_json);

            let route_start_offset = json[tag_end..].find('[').unwrap_or_default();
            let route_start = tag_end + route_start_offset;
            let route_end_offset = find_balanced_bracket_end(&json[route_start..]).unwrap_or(0);
            let route_end = route_start + route_end_offset;
            let routes_json = &json[route_start..route_end];
            let routes = extract_routes(routes_json);

            let assoc_start_offset = json[route_end..].find('[').unwrap_or_default();
            let assoc_start = route_end + assoc_start_offset;
            let assoc_end_offset = find_balanced_bracket_end(&json[assoc_start..]).unwrap_or(0);
            let assoc_end = assoc_start + assoc_end_offset;
            let assocs_json = &json[assoc_start..assoc_end];
            let associations = extract_associations(assocs_json);

            details.push(RouteTableDetail {
                name,
                id: id.to_string(),
                routes,
                associations,
            });

            search_start = assoc_end;
        } else {
            break;
        }
    }
    details
}

fn find_balanced_bracket_end(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        if c == '[' {
            depth += 1;
        } else if c == ']' {
            depth -= 1;
            if depth == 0 {
                return Some(i + 1);
            }
        }
    }
    None
}

fn extract_routes(json: &str) -> Vec<String> {
    let mut routes = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = json[search_start..].find("\"DestinationCidrBlock\": \"") {
        let block_start = search_start + pos;
        let obj_end = if let Some(end) = json[block_start..].find('}') {
            block_start + end
        } else {
            json.len()
        };

        let section = &json[block_start..obj_end];

        let dest = extract_json_value(section, "DestinationCidrBlock").unwrap_or_default();
        let gateway = extract_json_value(section, "GatewayId").unwrap_or_default();
        let nat = extract_json_value(section, "NatGatewayId").unwrap_or_default();
        let target = if !gateway.is_empty() {
            gateway
        } else if !nat.is_empty() {
            nat
        } else {
            "local".to_string()
        };
        let state = extract_json_value(section, "State").unwrap_or_else(|| "active".to_string());

        // Format: "dest|target|state"
        routes.push(format!("{}|{}|{}", dest, target, state));

        search_start = obj_end;
    }
    routes
}

fn extract_associations(json: &str) -> Vec<String> {
    let mut assocs = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = json[search_start..].find("\"SubnetId\": \"") {
        let block_start = search_start + pos;
        let obj_end = if let Some(end) = json[block_start..].find('}') {
            block_start + end
        } else {
            json.len()
        };

        let section = &json[block_start..obj_end];
        let subnet_id = extract_json_value(section, "SubnetId").unwrap_or_default();

        let name = subnet_adapter::get(&subnet_id);
        assocs.push(format!("{} ({})", name, subnet_id));

        search_start = obj_end;
    }
    assocs
}

pub fn list_eips() -> Vec<EipDetail> {
    let command = [
        "ec2",
        "describe-addresses",
        "--query",
        "Addresses[*].{AllocationId:AllocationId, PublicIp:PublicIp, AssociationId:AssociationId, InstanceId:InstanceId, PrivateIpAddress:PrivateIpAddress, Tags:Tags}",
        "--output",
        "json",
    ];
    let output = match cli_adapter::run(&command) {
        Some(o) => o,
        None => {
            tracing::warn!(
                command = %command.join(" "),
                "list_eips: no output from aws adapter"
            );
            return Vec::new();
        }
    };
    tracing::debug!(command = %command.join(" "), bytes = output.len(), "list_eips: aws adapter response");

    parse_eips_output(&output)
}

fn parse_eips_output(json: &str) -> Vec<EipDetail> {
    let mut details = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = json[search_start..].find("AllocationId") {
        let block_start = search_start + pos;
        let obj_start = json[..block_start].rfind('{').unwrap_or(0);

        let mut depth = 0;
        let mut obj_end = json.len();
        for (i, c) in json[obj_start..].char_indices() {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    obj_end = obj_start + i + 1;
                    break;
                }
            }
        }

        let section = &json[obj_start..obj_end];

        let public_ip = extract_json_value(section, "PublicIp").unwrap_or_default();
        let instance_id = extract_json_value(section, "InstanceId").unwrap_or_default();
        let private_ip = extract_json_value(section, "PrivateIpAddress").unwrap_or_default();

        let name = parse_name_tag(section);
        let display_name = if name.is_empty() {
            public_ip.clone()
        } else {
            name
        };

        details.push(EipDetail {
            name: display_name,
            public_ip,
            instance_id,
            private_ip,
        });

        search_start = obj_end;
    }

    details
}

pub fn get_network_detail(vpc_id: &str) -> Option<NetworkDetail> {
    let command = [
        "ec2",
        "describe-vpcs",
        "--vpc-ids",
        vpc_id,
        "--output",
        "json",
    ];
    let output = match cli_adapter::run(&command) {
        Some(output) => output,
        None => {
            tracing::warn!(
                vpc_id,
                command = %command.join(" "),
                "get_network_detail: no output from aws adapter"
            );
            return None;
        }
    };
    tracing::debug!(
        vpc_id,
        command = %command.join(" "),
        bytes = output.len(),
        "get_network_detail: aws adapter response"
    );

    let json = &output;
    let cidr = extract_json_value(json, "CidrBlock").unwrap_or_default();
    let state = extract_state(json);
    let tags = extract_tags(json);

    let name = tags
        .iter()
        .find(|(k, _)| k == "Name")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| vpc_id.to_string());

    let subnets = list_subnets(vpc_id);
    let igws = list_internet_gateways(vpc_id);
    let nats = list_nat_gateways(vpc_id);
    let route_tables = list_route_tables(vpc_id);
    let eips = list_eips();

    let dns_support = get_vpc_attribute(vpc_id, "enableDnsSupport");
    let dns_hostnames = get_vpc_attribute(vpc_id, "enableDnsHostnames");

    Some(NetworkDetail {
        name,
        id: vpc_id.to_string(),
        cidr,
        state,
        subnets,
        igws,
        nats,
        route_tables,
        eips,
        dns_support,
        dns_hostnames,
        tags,
    })
}

fn extract_state(json: &str) -> String {
    for state in ["available", "pending", "deleting", "deleted"] {
        if json.contains(&format!("\"State\": \"{}\"", state)) {
            return state.to_string();
        }
    }
    "unknown".to_string()
}

fn get_vpc_attribute(vpc_id: &str, attribute: &str) -> bool {
    let command = [
        "ec2",
        "describe-vpc-attribute",
        "--vpc-id",
        vpc_id,
        "--attribute",
        attribute,
        "--output",
        "json",
    ];
    let output = match cli_adapter::run(&command) {
        Some(json) => {
            tracing::debug!(
                vpc_id,
                attribute,
                command = %command.join(" "),
                bytes = json.len(),
                "get_vpc_attribute: aws adapter response"
            );
            json
        }
        None => {
            tracing::warn!(
                vpc_id,
                attribute,
                command = %command.join(" "),
                "get_vpc_attribute: no output from aws adapter"
            );
            return false;
        }
    };

    parse_vpc_attribute_response(&output, attribute)
}

fn parse_vpc_attribute_response(json: &str, attribute: &str) -> bool {
    let key = if attribute == "enableDnsSupport" {
        "EnableDnsSupport"
    } else {
        "EnableDnsHostnames"
    };
    if let Some(pos) = json.find(key) {
        let section = &json[pos..];
        if let Some(val_pos) = section.find("\"Value\": ") {
            let val_start = val_pos + 9;
            if section[val_start..].starts_with("true") {
                return true;
            }
        }
    }
    false
}

/// VPC 기본 정보만 조회 (name, cidr, state, tags)
type VpcInfo = (String, String, String, Vec<(String, String)>);

pub fn get_vpc_info(vpc_id: &str) -> Option<VpcInfo> {
    let command = [
        "ec2",
        "describe-vpcs",
        "--vpc-ids",
        vpc_id,
        "--output",
        "json",
    ];
    let output = match cli_adapter::run(&command) {
        Some(output) => output,
        None => {
            tracing::warn!(
                vpc_id,
                command = %command.join(" "),
                "get_vpc_info: no output from aws adapter"
            );
            return None;
        }
    };
    tracing::debug!(
        vpc_id,
        command = %command.join(" "),
        bytes = output.len(),
        "get_vpc_info: aws adapter response"
    );

    parse_vpc_info_output(&output, vpc_id)
}

fn parse_vpc_info_output(json: &str, vpc_id: &str) -> Option<VpcInfo> {
    let cidr = extract_json_value(json, "CidrBlock").unwrap_or_default();
    let state = extract_state(json);
    let tags = extract_tags(json);

    let name = tags
        .iter()
        .find(|(k, _)| k == "Name")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| vpc_id.to_string());

    Some((name, cidr, state, tags))
}

pub fn get_vpc_dns_support(vpc_id: &str) -> bool {
    get_vpc_attribute(vpc_id, "enableDnsSupport")
}

pub fn get_vpc_dns_hostnames(vpc_id: &str) -> bool {
    get_vpc_attribute(vpc_id, "enableDnsHostnames")
}

#[cfg(test)]
mod tests {
    use super::{
        EipDetail, NatDetail, NetworkDetail, RouteTableDetail, cli_adapter, extract_routes,
        find_balanced_bracket_end, get_network_detail, list_eips, list_nat_gateways,
        list_route_tables, list_subnets, list_vpcs, parse_eips_output, parse_internet_gateways,
        parse_nat_gateways_output, parse_route_tables, parse_subnets_output,
        parse_vpc_attribute_response, parse_vpc_info_output, subnet_adapter,
    };
    use crate::aws_cli::common::AwsResource;
    use crate::i18n::Language;
    use std::sync::{Mutex, OnceLock};

    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        match LOCK.get_or_init(|| Mutex::new(())).lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn sample_network_detail() -> NetworkDetail {
        NetworkDetail {
            name: "main-vpc".to_string(),
            id: "vpc-1111aaaa".to_string(),
            cidr: "10.0.0.0/16".to_string(),
            state: "available".to_string(),
            subnets: vec![AwsResource {
                name: "public-a".to_string(),
                id: "subnet-1111".to_string(),
                state: "available".to_string(),
                az: "ap-northeast-2a".to_string(),
                cidr: "10.0.1.0/24".to_string(),
            }],
            igws: vec![AwsResource {
                name: "igw-main".to_string(),
                id: "igw-1234".to_string(),
                state: "attached".to_string(),
                az: String::new(),
                cidr: String::new(),
            }],
            nats: vec![NatDetail {
                name: "nat-main".to_string(),
                id: "nat-1234".to_string(),
                state: "available".to_string(),
                connectivity_type: "public".to_string(),
                availability_mode: "regional".to_string(),
                auto_scaling_ips: "enabled".to_string(),
                auto_provision_zones: "enabled".to_string(),
                public_ip: "1.1.1.1".to_string(),
                allocation_id: "eipalloc-1234".to_string(),
                subnet_id: "subnet-1111".to_string(),
                tags: vec![("Env".to_string(), "prod".to_string())],
            }],
            route_tables: vec![RouteTableDetail {
                name: "rt-main".to_string(),
                id: "rtb-1234".to_string(),
                routes: vec!["0.0.0.0/0|igw-1234|active".to_string()],
                associations: vec!["public-a (subnet-1111)".to_string()],
            }],
            eips: vec![EipDetail {
                name: "eip-main".to_string(),
                public_ip: "1.1.1.1".to_string(),
                instance_id: "i-aaaa1111".to_string(),
                private_ip: String::new(),
            }],
            dns_support: true,
            dns_hostnames: true,
            tags: vec![("Name".to_string(), "main-vpc".to_string())],
        }
    }

    #[test]
    fn parse_internet_gateways_extracts_id_name_and_state() {
        let payload = r#"
            [
              ["igw-1234", [{"Key":"Name","Value":"igw-main"}], [{"VpcId":"vpc-1111","State":"available"}]],
              ["igw-5678", [], []]
            ]
        "#;
        let igws = parse_internet_gateways(payload);
        assert_eq!(igws.len(), 2);
        assert_eq!(igws[0].id, "igw-1234");
        assert_eq!(igws[0].state, "attached");
    }

    #[test]
    fn route_parsing_helpers_handle_nested_arrays() {
        assert_eq!(find_balanced_bracket_end("[[]]"), Some(4));
        assert_eq!(find_balanced_bracket_end("[[]"), None);

        let routes = extract_routes(
            r#"[{"DestinationCidrBlock": "0.0.0.0/0", "GatewayId": "igw-1234", "State": "active"}]"#,
        );
        assert_eq!(routes, vec!["0.0.0.0/0|igw-1234|active".to_string()]);
    }

    #[test]
    fn parse_route_tables_extracts_table_id() {
        let payload = r#"
            [
              ["rtb-1234", [{"Key":"Name","Value":"rt-main"}],
                [{"DestinationCidrBlock":"0.0.0.0/0","GatewayId":"igw-1234","State":"active"}],
                [{"SubnetId":"subnet-1111"}]
              ]
            ]
        "#;
        let tables = parse_route_tables(payload);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].id, "rtb-1234");
    }

    #[test]
    fn network_markdown_contains_sections_and_mermaid() {
        let md = sample_network_detail().to_markdown(Language::English);
        assert!(md.contains("## Network"));
        assert!(md.contains("Internet Gateway"));
        assert!(md.contains("NAT Gateway"));
        assert!(md.contains("```mermaid"));
    }

    #[test]
    fn parse_internet_gateways_marks_detached_when_attachment_is_missing() {
        let payload = r#"
            [
              ["igw-1111", [{"Key":"Name","Value":"igw-a"}], []],
              ["invalid-id", [], []]
            ]
        "#;
        let igws = parse_internet_gateways(payload);
        assert_eq!(igws.len(), 1);
        assert_eq!(igws[0].id, "igw-1111");
        assert_eq!(igws[0].state, "detached");
    }

    #[test]
    fn extract_routes_uses_nat_or_local_when_gateway_is_missing() {
        let routes = extract_routes(
            r#"
            [
              {"DestinationCidrBlock": "0.0.0.0/0", "NatGatewayId": "nat-1111", "State": "active"},
              {"DestinationCidrBlock": "10.0.0.0/16", "State": "active"}
            ]
            "#,
        );
        assert_eq!(routes[0], "0.0.0.0/0|nat-1111|active");
        assert_eq!(routes[1], "10.0.0.0/16|local|active");
    }

    #[test]
    fn parse_route_tables_supports_multiple_tables() {
        let payload = r#"
            [
              ["rtb-1111", [{"Key":"Name","Value":"rt-a"}],
                [{"DestinationCidrBlock":"0.0.0.0/0","GatewayId":"igw-1","State":"active"}],
                [{"SubnetId":"subnet-1111"}]
              ],
              ["rtb-2222", [{"Key":"Name","Value":"rt-b"}],
                [{"DestinationCidrBlock":"10.0.0.0/16","State":"active"}],
                []
              ]
            ]
        "#;
        let tables = parse_route_tables(payload);
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].id, "rtb-1111");
        assert_eq!(tables[1].id, "rtb-2222");
    }

    #[test]
    fn network_markdown_zonal_nat_path_renders_subnet_and_private_assoc() {
        let detail = NetworkDetail {
            name: "vpc-zonal".to_string(),
            id: "vpc-zonal".to_string(),
            cidr: "10.1.0.0/16".to_string(),
            state: "available".to_string(),
            subnets: vec![AwsResource {
                name: "private-a".to_string(),
                id: "subnet-z1".to_string(),
                state: "available".to_string(),
                az: "ap-northeast-2a".to_string(),
                cidr: "10.1.1.0/24".to_string(),
            }],
            igws: vec![],
            nats: vec![NatDetail {
                name: "nat-zonal".to_string(),
                id: "nat-z1".to_string(),
                state: "available".to_string(),
                connectivity_type: "private".to_string(),
                availability_mode: String::new(),
                auto_scaling_ips: String::new(),
                auto_provision_zones: String::new(),
                public_ip: String::new(),
                allocation_id: String::new(),
                subnet_id: "subnet-z1".to_string(),
                tags: vec![],
            }],
            route_tables: vec![],
            eips: vec![EipDetail {
                name: "eip-private".to_string(),
                public_ip: "52.0.0.9".to_string(),
                instance_id: String::new(),
                private_ip: "10.1.1.11".to_string(),
            }],
            dns_support: true,
            dns_hostnames: false,
            tags: vec![],
        };

        let md = detail.to_markdown(Language::English);
        assert!(md.contains("NAT Gateway"));
        assert!(md.contains("private-a - subnet-z1"));
        assert!(md.contains("Private"));
        assert!(md.contains("Private IP: 10.1.1.11"));
    }

    #[test]
    fn subnet_nat_eip_and_vpc_attribute_parsers_handle_fixture_payloads() {
        let subnets_payload = r#"
        {
          "Subnets": [
            {
              "SubnetId": "subnet-a",
              "VpcId": "vpc-1",
              "CidrBlock": "10.0.1.0/24",
              "AvailabilityZone": "ap-northeast-2a",
              "State": "available",
              "MapPublicIpOnLaunch": true,
              "AvailableIpAddressCount": 251,
              "Tags": [{"Key":"Name","Value":"public-a"}]
            },
            {
              "SubnetId": "subnet-b",
              "VpcId": "vpc-2",
              "CidrBlock": "10.1.1.0/24",
              "AvailabilityZone": "ap-northeast-2b",
              "State": "available",
              "Tags": []
            }
          ]
        }
        "#;
        let subnets = parse_subnets_output(subnets_payload, "vpc-1");
        assert_eq!(subnets.len(), 1);
        assert_eq!(subnets[0].id, "subnet-a");
        assert_eq!(subnets[0].name, "public-a");

        let nat_payload = r#"
        {
          "NatGateways": [
            {
              "NatGatewayId": "nat-1",
              "SubnetId": "subnet-a",
              "State": "available",
              "ConnectivityType": "public",
              "AvailabilityMode": "regional",
              "AutoScalingIps": "enabled",
              "AutoProvisionZones": "enabled",
              "NatGatewayAddresses": [
                {"AllocationId":"eipalloc-1","PublicIp":"3.3.3.3","PrivateIp":"10.0.1.10"}
              ],
              "Tags": [
                {"Key":"Name","Value":"nat-main"},
                {"Key":"Env","Value":"prod"}
              ]
            },
            {
              "NatGatewayId": "nat-del",
              "State": "deleted",
              "NatGatewayAddresses": [],
              "Tags": []
            }
          ]
        }
        "#;
        let nats = parse_nat_gateways_output(nat_payload);
        assert_eq!(nats.len(), 1);
        assert_eq!(nats[0].id, "nat-1");
        assert_eq!(nats[0].allocation_id, "eipalloc-1");
        assert_eq!(nats[0].tags[0], ("Env".to_string(), "prod".to_string()));

        let eip_payload = r#"
        [
          {"AllocationId": "eipalloc-1", "PublicIp": "3.3.3.3", "InstanceId": "i-1", "PrivateIpAddress": "10.0.1.10", "Tags": [{"Key": "Name", "Value": "edge-eip"}]},
          {"AllocationId": "eipalloc-2", "PublicIp": "3.3.3.4", "AssociationId": "eipassoc-2", "Tags": []}
        ]
        "#;
        let eips = parse_eips_output(eip_payload);
        assert_eq!(eips.len(), 2);
        assert_eq!(eips[0].name, "edge-eip");
        assert_eq!(eips[1].name, "3.3.3.4");

        let attr_true = r#"{"EnableDnsSupport":{"Value": true}}"#;
        let attr_false = r#"{"EnableDnsHostnames":{"Value": false}}"#;
        assert!(parse_vpc_attribute_response(attr_true, "enableDnsSupport"));
        assert!(!parse_vpc_attribute_response(
            attr_false,
            "enableDnsHostnames"
        ));
    }

    #[test]
    fn parse_vpc_info_output_prefers_name_tag_and_extracts_defaults() {
        let payload = r#"
        {
          "Vpcs": [
            {
              "VpcId": "vpc-abc",
              "CidrBlock": "10.2.0.0/16",
              "State": "available",
              "Tags":[
                {"Key": "Name", "Value": "core-vpc"},
                {"Key": "Env", "Value": "prod"}
              ]
            }
          ]
        }
        "#;
        let info = parse_vpc_info_output(payload, "vpc-abc").expect("vpc info");
        assert_eq!(info.0, "core-vpc");
        assert_eq!(info.1, "10.2.0.0/16");
        assert_eq!(info.2, "available");
        assert!(info.3.iter().any(|(k, v)| k == "Env" && v == "prod"));
    }

    #[test]
    fn top_level_vpc_functions_use_mocked_cli_outputs() {
        let _guard = test_lock();
        cli_adapter::clear();
        subnet_adapter::clear();

        subnet_adapter::set("subnet-1111", "public-a");

        cli_adapter::set(
            &[
                "ec2",
                "describe-vpcs",
                "--query",
                "Vpcs[*].[VpcId,Tags]",
                "--output",
                "json",
            ],
            Some(r#"[["vpc-1111",[{"Key": "Name", "Value": "main-vpc"}]]]"#),
        );
        cli_adapter::set(
            &["ec2", "describe-subnets", "--output", "json"],
            Some(
                r#"
                {
                  "Subnets": [
                    {
                      "SubnetId": "subnet-1111",
                      "VpcId": "vpc-1111",
                      "CidrBlock": "10.0.1.0/24",
                      "AvailabilityZone": "ap-northeast-2a",
                      "State": "available",
                      "Tags": [{"Key":"Name","Value":"public-a"}]
                    }
                  ]
                }
                "#,
            ),
        );
        cli_adapter::set(
            &[
                "ec2",
                "describe-internet-gateways",
                "--filters",
                "Name=attachment.vpc-id,Values=vpc-1111",
                "--query",
                "InternetGateways[*].[InternetGatewayId,Tags,Attachments]",
                "--output",
                "json",
            ],
            Some(
                r#"
                [
                  ["igw-1234", [{"Key": "Name", "Value": "igw-main"}], [{"VpcId": "vpc-1111", "State": "available"}]]
                ]
                "#,
            ),
        );
        cli_adapter::set(
            &[
                "ec2",
                "describe-nat-gateways",
                "--filter",
                "Name=vpc-id,Values=vpc-1111",
                "--output",
                "json",
            ],
            Some(
                r#"
                {
                  "NatGateways": [
                    {
                      "NatGatewayId": "nat-1234",
                      "SubnetId": "subnet-1111",
                      "State": "available",
                      "ConnectivityType": "public",
                      "NatGatewayAddresses": [{"AllocationId": "eipalloc-1234", "PublicIp": "52.0.0.1"}],
                      "Tags": [{"Key": "Name", "Value": "nat-main"}]
                    }
                  ]
                }
                "#,
            ),
        );
        cli_adapter::set(
            &[
                "ec2",
                "describe-route-tables",
                "--filters",
                "Name=vpc-id,Values=vpc-1111",
                "--query",
                "RouteTables[*].[RouteTableId,Tags,Routes,Associations]",
                "--output",
                "json",
            ],
            Some(
                r#"
                [
                  [
                    "rtb-1234",
                    [{"Key": "Name", "Value": "rt-main"}],
                    [{"DestinationCidrBlock": "0.0.0.0/0", "GatewayId": "igw-1234", "State": "active"}],
                    [{"SubnetId": "subnet-1111"}]
                  ]
                ]
                "#,
            ),
        );
        cli_adapter::set(
            &[
                "ec2",
                "describe-addresses",
                "--query",
                "Addresses[*].{AllocationId:AllocationId, PublicIp:PublicIp, AssociationId:AssociationId, InstanceId:InstanceId, PrivateIpAddress:PrivateIpAddress, Tags:Tags}",
                "--output",
                "json",
            ],
            Some(
                r#"
                [
                  {"AllocationId": "eipalloc-1234", "PublicIp": "52.0.0.1", "InstanceId": "i-1234", "Tags": [{"Key": "Name", "Value": "edge-eip"}]}
                ]
                "#,
            ),
        );

        cli_adapter::set(
            &[
                "ec2",
                "describe-vpcs",
                "--vpc-ids",
                "vpc-1111",
                "--output",
                "json",
            ],
            Some(
                r#"
                {
                  "Vpcs": [
                    {
                      "VpcId": "vpc-1111",
                      "CidrBlock": "10.0.0.0/16",
                      "State": "available",
                      "Tags": [{"Key": "Name", "Value": "main-vpc"}]
                    }
                  ]
                }
                "#,
            ),
        );
        cli_adapter::set(
            &[
                "ec2",
                "describe-vpc-attribute",
                "--vpc-id",
                "vpc-1111",
                "--attribute",
                "enableDnsSupport",
                "--output",
                "json",
            ],
            Some(r#"{"EnableDnsSupport":{"Value": true}}"#),
        );
        cli_adapter::set(
            &[
                "ec2",
                "describe-vpc-attribute",
                "--vpc-id",
                "vpc-1111",
                "--attribute",
                "enableDnsHostnames",
                "--output",
                "json",
            ],
            Some(r#"{"EnableDnsHostnames":{"Value": true}}"#),
        );

        let vpcs = list_vpcs();
        assert_eq!(vpcs.len(), 1);
        assert_eq!(vpcs[0].id, "vpc-1111");

        let subnets = list_subnets("vpc-1111");
        assert_eq!(subnets.len(), 1);
        assert_eq!(subnets[0].name, "public-a");

        let nats = list_nat_gateways("vpc-1111");
        assert_eq!(nats.len(), 1);
        assert_eq!(nats[0].id, "nat-1234");

        let routes = list_route_tables("vpc-1111");
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].id, "rtb-1234");
        assert_eq!(routes[0].associations[0], "public-a (subnet-1111)");

        let eips = list_eips();
        assert_eq!(eips.len(), 1);
        assert_eq!(eips[0].name, "edge-eip");

        let detail = get_network_detail("vpc-1111").expect("network detail");
        assert_eq!(detail.id, "vpc-1111");
        assert_eq!(detail.name, "main-vpc");
        assert!(detail.dns_support);
        assert!(detail.dns_hostnames);
    }
}
