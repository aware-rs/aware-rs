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

mod show;

#[derive(Debug, StructOpt)]
struct Aware {
    #[structopt(long, short)]
    region: Vec<String>,
    #[structopt(long, short)]
    vpc: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), ec2::Error> {
    let aware = Aware::from_args();

    let regions = if aware.region.is_empty() {
        get_all_regions().await?
    } else {
        aware.region
    };
    collect(regions, aware.vpc).await
}

fn filter(key: &str, value: &str) -> ec2::model::Filter {
    ec2::model::Filter::builder()
        .name(key)
        .values(value)
        .build()
}

async fn collect(regions: Vec<String>, vpc: Vec<String>) -> Result<(), ec2::Error> {
    let regioned_clients = regions
        .into_iter()
        .map(ec2::Region::new)
        .map(RegionProviderChain::first_try);

    for region in regioned_clients {
        let shared_config = aws_config::from_env().region(region).load().await;
        let name = format!("AWS Region {:?}", shared_config.region().id_and_name());
        let mut tree = ptree::TreeBuilder::new(name);
        let client = ec2::Client::new(&shared_config);

        let vpcs = get_all_vpc_from_region(&client).await?;
        let vpcs = if vpc.is_empty() {
            vpcs
        } else {
            vpcs.into_iter().filter(|v| vpc.contains(&v.id())).collect()
        };

        let progress = indicatif::ProgressBar::new(vpcs.len() as u64).with_style(
            indicatif::ProgressStyle::default_bar().template("{msg} {wide_bar} {pos}/{len}"),
        );
        for vpc in vpcs {
            progress.set_message(vpc.vpc_id.clone().unwrap_or_default());
            let _ = collect_vpc(&client, &vpc, &mut tree).await?;
            progress.inc(1);
        }
        progress.finish();
        let tree = tree.build();
        ptree::print_tree(&tree).expect("Failed to print tree");
    }

    Ok(())
}

async fn get_all_regions() -> Result<Vec<String>, ec2::Error> {
    let shared_config = aws_config::load_from_env().await;
    let client = ec2::Client::new(&shared_config);

    let regions = client
        .describe_regions()
        .all_regions(true)
        .send()
        .await?
        .regions
        .unwrap_or_default()
        .into_iter()
        .filter_map(|region| region.region_name)
        .collect();

    Ok(regions)
}

async fn get_all_vpc_from_region(client: &ec2::Client) -> Result<Vec<ec2::model::Vpc>, ec2::Error> {
    let vpcs = client
        .describe_vpcs()
        .send()
        .await?
        .vpcs
        .unwrap_or_default()
        .into_iter()
        .collect();
    Ok(vpcs)
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

async fn collect_vpc(
    client: &ec2::Client,
    vpc: &ec2::model::Vpc,
    ptree: &mut ptree::TreeBuilder,
) -> Result<(), ec2::Error> {
    ptree.begin_child(vpc.id_and_name());

    if let Some(ref vpc) = vpc.vpc_id {
        macro_rules! collect {
            ($collector:ident, $title:expr) => {{
                $collector(client, vpc)
                    .await
                    .map(|resources| add_children(ptree, $title, resources))?
            }};
        }
        collect!(subnets, "Subnets");
        collect!(instances, "Instances");
        collect!(internet_gateways, "Internet Gateway");
        collect!(route_tables, "Route Tables");

        // let network_acls = get_all_network_acls_from_vpc(client, vpc).await?;
        // resources.extend(network_acls);

        // let vpc_peering_connections = get_all_vpc_peering_connections_from_vpc(client, vpc).await?;
        // resources.extend(vpc_peering_connections);

        // let vpc_endpoints = get_all_vpc_endpoints_from_vpc(client, vpc).await?;
        // resources.extend(vpc_endpoints);

        // let nat_gateways = get_all_nat_gateways_from_vpc(client, vpc).await?;
        // resources.extend(nat_gateways);

        // let security_groups = get_all_security_groups_from_vpc(client, vpc).await?;
        // resources.extend(security_groups);

        // let vpn_connections = get_all_vpn_connections_from_vpc(client, vpc).await?;
        // resources.extend(vpn_connections);

        // let vpn_gateways = get_all_vpn_gateways_from_vpc(client, &vpc).await?;
        // resources.extend(vpn_gateways);
    }
    ptree.end_child();
    Ok(())
}

async fn internet_gateways(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<ec2::model::InternetGateway>, ec2::Error> {
    let vpc = filter("attachment.vpc-id", vpc);
    let igw = client
        .describe_internet_gateways()
        .filters(vpc)
        .send()
        .await?
        .internet_gateways
        .unwrap_or_default();
    Ok(igw)
}

async fn subnets(client: &ec2::Client, vpc: &str) -> Result<Vec<ec2::model::Subnet>, ec2::Error> {
    let vpc = filter("vpc-id", vpc);
    let subnets = client
        .describe_subnets()
        .filters(vpc)
        .send()
        .await?
        .subnets
        .unwrap_or_default();
    Ok(subnets)
}

async fn route_tables(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<ec2::model::RouteTable>, ec2::Error> {
    let vpc = filter("vpc-id", vpc);
    let tables = client
        .describe_route_tables()
        .filters(vpc)
        .send()
        .await?
        .route_tables
        .unwrap_or_default();
    Ok(tables)
}

async fn instances(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<ec2::model::Instance>, ec2::Error> {
    let vpc = filter("vpc-id", vpc);
    let instances = client
        .describe_instances()
        .filters(vpc)
        .send()
        .await?
        .reservations
        .unwrap_or_default()
        .into_iter()
        .flat_map(|reservation| reservation.instances.unwrap_or_default())
        .collect();
    Ok(instances)
}
