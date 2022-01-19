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

use std::env;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2 as ec2;
use clap::{Parser, Subcommand};

use show::Show;

mod aws;
mod show;

#[derive(Debug, Parser)]
struct Aware {
    #[clap(
        help = "Explore resources from this region / these regions",
        long,
        short,
        global = true
    )]
    region: Vec<String>,
    #[clap(subcommand)]
    service: AwsService,
}

#[derive(Debug, Subcommand)]
pub(crate) enum AwsService {
    #[clap(name = "ec2", about = "Explore EC2 resources")]
    Ec2 {
        #[clap(long, short)]
        vpc: Vec<String>,
    },
    #[clap(name = "cf", about = "Explore CloudFormation resources")]
    CloudFormation {
        #[clap(help = "Filter by given stack name", long)]
        stack: Vec<String>,
        #[clap(help = "Filter by given stack status", long)]
        status: Vec<aws::cf::model::StackStatus>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let aware = Aware::parse();

    let regions = if aware.region.is_empty() {
        get_all_regions().await?
    } else {
        aware.region
    };

    match aware.service {
        AwsService::Ec2 { vpc } => collect_ec2(regions, vpc).await,
        AwsService::CloudFormation { stack, status } => collect_cf(regions, stack, status).await,
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

async fn collect_ec2(regions: Vec<String>, vpc: Vec<String>) -> anyhow::Result<()> {
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
        let mut ec2 = aws::Ec2Resources::new(client);
        progress.set_message("Collecting VPCs");
        ec2.collect_vpcs(&vpc).await?;
        progress.inc(1);

        ec2.collect(&progress).await?;

        progress.finish();

        ec2.trees().for_each(|tree| {
            println!();
            ptree::print_tree(&tree).expect("Failed to print tree");
        });
    }

    Ok(())
}

async fn collect_cf(
    regions: Vec<String>,
    stack: Vec<String>,
    status: Vec<aws::cf::model::StackStatus>,
) -> anyhow::Result<()> {
    let regioned_clients = regions
        .into_iter()
        .map(ec2::Region::new)
        .map(RegionProviderChain::first_try);
    let statuses = aws::cf::adjust_stack_statuses(status);

    for region in regioned_clients {
        let shared_config = aws_config::from_env().region(region).load().await;
        let region = format!("AWS Region {:?}", shared_config.region().id_and_name());
        // let client = cf::Client::new(&shared_config);

        let progress = indicatif::ProgressBar::new(1)
            .with_style(indicatif::ProgressStyle::default_bar().template(
            "[{pos:>3}/{len:>3} {prefix}] {msg:24!} {wide_bar} [{elapsed}/{duration} ETA {eta}]",
        ));
        progress.set_prefix(region.clone());
        let mut cf = aws::CfResources::new(&shared_config);
        progress.set_message("Collecting stacks");
        cf.collect_stacks(&stack, &statuses).await?;
        progress.inc(1);

        cf.collect_stack_resources(&progress).await?;

        progress.finish();

        cf.trees().for_each(|tree| {
            println!();
            ptree::print_tree(&tree).expect("Failed to print tree");
        });
    }

    Ok(())
}
