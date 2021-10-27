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

fn filter(key: &str, value: &str) -> ec2::model::Filter {
    ec2::model::Filter::builder()
        .name(key)
        .values(value)
        .build()
}
