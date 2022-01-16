#![cfg_attr(feature = "pedantic", warn(clippy::pedantic))]
#![warn(clippy::use_self)]
#![warn(clippy::map_flatten)]
#![warn(clippy::map_unwrap_or)]
#![warn(deprecated_in_future)]
#![warn(future_incompatible)]
#![warn(noop_method_call)]
#![warn(unreachable_pub)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(unused)]
#![deny(warnings)]

// #!/bin/bash
// vpc="vpc-xxxxxxxxxxxxx"
// aws ec2 describe-internet-gateways --filters 'Name=attachment.vpc-id,Values='$vpc | grep InternetGatewayId
// aws ec2 describe-subnets --filters 'Name=vpc-id,Values='$vpc | grep SubnetId
// aws ec2 describe-route-tables --filters 'Name=vpc-id,Values='$vpc | grep RouteTableId
// aws ec2 describe-network-acls --filters 'Name=vpc-id,Values='$vpc | grep NetworkAclId
// aws ec2 describe-vpc-peering-connections --filters 'Name=requester-vpc-info.vpc-id,Values='$vpc | grep VpcPeeringConnectionId
// aws ec2 describe-vpc-endpoints --filters 'Name=vpc-id,Values='$vpc | grep VpcEndpointId
// aws ec2 describe-nat-gateways --filter 'Name=vpc-id,Values='$vpc | grep NatGatewayId
// aws ec2 describe-security-groups --filters 'Name=vpc-id,Values='$vpc | grep GroupId
// aws ec2 describe-instances --filters 'Name=vpc-id,Values='$vpc | grep InstanceId
// aws ec2 describe-vpn-connections --filters 'Name=vpc-id,Values='$vpc | grep VpnConnectionId
// aws ec2 describe-vpn-gateways --filters 'Name=attachment.vpc-id,Values='$vpc | grep VpnGatewayId
// aws ec2 describe-network-interfaces --filters 'Name=vpc-id,Values='$vpc | grep NetworkInterfaceId

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2 as ec2;
use structopt::StructOpt;

use show::Show;

mod aws;
mod show;

#[derive(Debug, StructOpt)]
struct Aware {
    #[structopt(
        help = "Explore resources from this region / these regions",
        long,
        short,
        global = true
    )]
    region: Vec<String>,
    #[structopt(subcommand)]
    service: AwsService,
}

#[derive(Debug, StructOpt)]
pub(crate) enum AwsService {
    #[structopt(name = "ec2", about = "Explore EC2 resources")]
    Ec2 {
        #[structopt(long, short)]
        vpc: Vec<String>,
    },
    #[structopt(name = "cf", about = "Explore CloudFormation resources")]
    CloudFormation,
}

#[tokio::main]
async fn main() -> Result<(), ec2::Error> {
    let aware = Aware::from_args();

    let regions = if aware.region.is_empty() {
        get_all_regions().await?
    } else {
        aware.region
    };

    match aware.service {
        AwsService::Ec2 { vpc } => collect_ec2(regions, vpc).await,
        AwsService::CloudFormation => todo!(),
    }
}

async fn get_all_regions() -> Result<Vec<String>, ec2::Error> {
    let shared_config = aws_config::load_from_env().await;
    let client = ec2::Client::new(&shared_config);

    let regions = aws::regions(&client)
        .await?
        .into_iter()
        .filter_map(|region| region.region_name)
        .collect();

    Ok(regions)
}

async fn collect_ec2(regions: Vec<String>, vpc: Vec<String>) -> Result<(), ec2::Error> {
    let regioned_clients = regions
        .into_iter()
        .map(ec2::Region::new)
        .map(RegionProviderChain::first_try);

    for region in regioned_clients {
        let shared_config = aws_config::from_env().region(region).load().await;
        let region = format!("AWS Region {:?}", shared_config.region().id_and_name());
        let client = ec2::Client::new(&shared_config);

        let progress = indicatif::ProgressBar::new(1).with_style(
            indicatif::ProgressStyle::default_bar().template(
                "[{pos}/{len} {prefix}] {msg:24} {wide_bar} [{elapsed}/{duration} ETA {eta}]",
            ),
        );
        progress.set_prefix(region.clone());
        let mut aws = aws::Ec2Resources::new(client);
        progress.set_message("Collecting VPCs");
        aws.collect_vpcs(&vpc).await?;
        progress.inc(1);

        aws.collect(&progress).await?;

        progress.finish();

        aws.vpcs()
            .iter()
            .map(|vpc| vpc_tree(&aws, vpc))
            .for_each(|tree| {
                println!();
                ptree::print_tree(&tree).expect("Failed to print tree");
            });
    }

    Ok(())
}

fn vpc_tree(aws: &aws::Ec2Resources, vpc: &ec2::model::Vpc) -> ptree::item::StringItem {
    let mut tree = ptree::TreeBuilder::new(vpc.id_and_name());
    let tree = &mut tree;
    let vpc_id = vpc.id();
    add_children(tree, "Subnets", aws.subnets(&vpc_id));
    add_children(tree, "Instances", aws.instances(&vpc_id));
    add_children(tree, "Internet Gateways", aws.internet_gateways(&vpc_id));
    add_children(tree, "Route Tables", aws.route_tables(&vpc_id));
    add_children(tree, "Network ACLs", aws.network_acls(&vpc_id));
    add_children(tree, "VPC Peering Connections", aws.vpc_peerings(&vpc_id));
    add_children(tree, "VPC Endpoints", aws.vpc_endpoints(&vpc_id));
    add_children(tree, "NAT Gateways", aws.nat_gateways(&vpc_id));
    add_children(tree, "Security Groups", aws.security_groups(&vpc_id));
    add_children(tree, "VPN Connections", aws.vpn_connections(&vpc_id));
    add_children(tree, "VPN Gateways", aws.vpn_gateways(&vpc_id));
    add_children(tree, "Network Interfaces", aws.network_interfaces(&vpc_id));
    tree.build()
}

fn add_children(ptree: &mut ptree::TreeBuilder, title: impl ToString, resources: Vec<impl Show>) {
    if !resources.is_empty() {
        ptree.begin_child(title.to_string());
        resources.into_iter().for_each(|resource| {
            ptree.add_empty_child(resource.id_and_name());
        });
        ptree.end_child();
    }
}
