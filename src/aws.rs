use aws_sdk_ec2 as ec2;

pub(super) async fn regions(client: &ec2::Client) -> Result<Vec<ec2::model::Region>, ec2::Error> {
    let regions = client
        .describe_regions()
        .all_regions(true)
        .send()
        .await?
        .regions
        .unwrap_or_default();
    Ok(regions)
}

pub(super) async fn vpcs(client: &ec2::Client) -> Result<Vec<ec2::model::Vpc>, ec2::Error> {
    let vpcs = client
        .describe_vpcs()
        .send()
        .await?
        .vpcs
        .unwrap_or_default();
    Ok(vpcs)
}

pub(super) async fn internet_gateways(
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

pub(super) async fn subnets(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<ec2::model::Subnet>, ec2::Error> {
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

pub(super) async fn route_tables(
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

pub(super) async fn instances(
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

pub(super) async fn network_acls(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<ec2::model::NetworkAcl>, ec2::Error> {
    let vpc = filter("vpc-id", vpc);
    let acls = client
        .describe_network_acls()
        .filters(vpc)
        .send()
        .await?
        .network_acls
        .unwrap_or_default();
    Ok(acls)
}

pub(super) async fn vpc_peering_connections(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<ec2::model::VpcPeeringConnection>, ec2::Error> {
    let vpc = filter("requester-vpc-info.vpc-id", vpc);
    let connections = client
        .describe_vpc_peering_connections()
        .filters(vpc)
        .send()
        .await?
        .vpc_peering_connections
        .unwrap_or_default();
    Ok(connections)
}

pub(super) async fn vpc_endpoints(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<ec2::model::VpcEndpoint>, ec2::Error> {
    let vpc = filter("vpc-id", vpc);
    let endpoints = client
        .describe_vpc_endpoints()
        .filters(vpc)
        .send()
        .await?
        .vpc_endpoints
        .unwrap_or_default();
    Ok(endpoints)
}

pub(super) async fn nat_gateways(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<ec2::model::NatGateway>, ec2::Error> {
    let vpc = filter("vpc-id", vpc);
    let gateways = client
        .describe_nat_gateways()
        .filter(vpc)
        .send()
        .await?
        .nat_gateways
        .unwrap_or_default();
    Ok(gateways)
}

pub(super) async fn security_groups(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<ec2::model::SecurityGroup>, ec2::Error> {
    let vpc = filter("vpc-id", vpc);
    let sgroups = client
        .describe_security_groups()
        .filters(vpc)
        .send()
        .await?
        .security_groups
        .unwrap_or_default();
    Ok(sgroups)
}

pub(super) async fn vpn_connections(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<ec2::model::VpnConnection>, ec2::Error> {
    let vpc = filter("vpc-id", vpc);
    let vpn_connections = client
        .describe_vpn_connections()
        .filters(vpc)
        .send()
        .await?
        .vpn_connections
        .unwrap_or_default();
    Ok(vpn_connections)
}

pub(super) async fn vpn_gateways(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<ec2::model::VpnGateway>, ec2::Error> {
    let vpc = filter("attachment.vpc-id", vpc);
    let vpn_gateways = client
        .describe_vpn_gateways()
        .filters(vpc)
        .send()
        .await?
        .vpn_gateways
        .unwrap_or_default();
    Ok(vpn_gateways)
}

pub(super) async fn network_interfaces(
    client: &ec2::Client,
    vpc: &str,
) -> Result<Vec<ec2::model::NetworkInterface>, ec2::Error> {
    let vpc = filter("vpc-id", vpc);
    let ifaces = client
        .describe_network_interfaces()
        .filters(vpc)
        .send()
        .await?
        .network_interfaces
        .unwrap_or_default();
    Ok(ifaces)
}

fn filter(key: &str, value: &str) -> ec2::model::Filter {
    ec2::model::Filter::builder()
        .name(key)
        .values(value)
        .build()
}
