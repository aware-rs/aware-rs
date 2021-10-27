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
}

#[tokio::main]
async fn main() -> Result<(), ec2::Error> {
    let aware = Aware::from_args();

    let regions = if aware.region.is_empty() {
        get_all_regions().await?
    } else {
        aware.region
    };
    scrape(regions).await
}

async fn scrape(regions: Vec<String>) -> Result<(), ec2::Error> {
    let regioned_clients = regions
        .into_iter()
        .map(ec2::Region::new)
        .map(RegionProviderChain::first_try);

    for region in regioned_clients {
        println!("{:#?}", region.region().await);
        let config = aws_config::from_env().region(region).load().await;
        let client = ec2::Client::new(&config);

        let vpcs = get_all_vpc_from_region(&client).await?;
        for vpc in vpcs {
            println!("{}", vpc.id_and_name());
            let resources = scrape_vpc(&client, &vpc).await?;
            println!("{:#?}", resources);
        }
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

async fn scrape_vpc(
    client: &ec2::Client,
    vpc: &ec2::model::Vpc,
) -> Result<Vec<String>, ec2::Error> {
    let mut resources = vec![];

    if let Some(ref vpc) = vpc.vpc_id {
        let internet_gateways = scrape_internet_gateways(client, vpc).await?;
        resources.extend(internet_gateways);

        // let subnets = get_all_subnets_from_vpc(client, vpc).await?;
        // resources.extend(subnets);

        // let route_tables = get_all_route_tables_from_vpc(client, vpc).await?;
        // resources.extend(route_tables);

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

        // let instances = get_all_instances_from_vpc(client, vpc).await?;
        // resources.extend(instances);

        // let vpn_connections = get_all_vpn_connections_from_vpc(client, vpc).await?;
        // resources.extend(vpn_connections);

        // let vpn_gateways = get_all_vpn_gateways_from_vpc(client, &vpc).await?;
        // resources.extend(vpn_gateways);
    }

    Ok(resources)
}

async fn scrape_internet_gateways(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<String>, ec2::Error> {
    let vpc = ec2::model::Filter::builder()
        .name("attachment.vpc-id")
        .values(vpc)
        .build();
    let igw = client
        .describe_internet_gateways()
        .filters(vpc)
        .send()
        .await?
        .internet_gateways
        .unwrap_or_default()
        .into_iter()
        .map(|igw| igw.id_and_name())
        .collect();
    Ok(igw)
}
