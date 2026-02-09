mod asg;
mod common;
mod ec2;
mod ecr;
pub(crate) mod iam;
mod load_balancer;
mod security_group;
mod vpc;

// Re-export common types
pub use common::{AwsResource, check_aws_login, set_region};

// Re-export EC2 types and functions
#[allow(unused_imports)]
pub use ec2::{Ec2Detail, VolumeDetail, get_instance_detail, list_instances};

// Re-export VPC types and functions
#[allow(unused_imports)]
pub use vpc::{
    EipDetail, NatDetail, NetworkDetail, RouteTableDetail, get_network_detail,
    get_vpc_dns_hostnames, get_vpc_dns_support, get_vpc_info, list_eips, list_internet_gateways,
    list_nat_gateways, list_route_tables, list_subnets, list_vpcs,
};

// Re-export Security Group types and functions
#[allow(unused_imports)]
pub use security_group::{
    SecurityGroupDetail, SecurityRule, get_security_group_detail, list_security_groups,
};

// Re-export Load Balancer types and functions
#[allow(unused_imports)]
pub use load_balancer::{
    ListenerInfo, LoadBalancerDetail, TargetGroupInfo, TargetInfo, get_load_balancer_detail,
    list_load_balancers,
};

// Re-export ECR types and functions
pub use ecr::{EcrDetail, get_ecr_detail, list_ecr_repositories};

// Re-export ASG types and functions
#[allow(unused_imports)]
pub use asg::{AsgDetail, get_asg_detail, list_auto_scaling_groups};
