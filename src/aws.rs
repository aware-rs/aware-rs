use aws_sdk_ec2 as ec2;

use impls::Optionally;

mod impls;

#[derive(Debug)]
pub(crate) struct Ec2Resources {
    client: ec2::Client,
    vpcs: Vec<ec2::model::Vpc>,
    subnets: Vec<ec2::model::Subnet>,                      // 1
    instances: Vec<ec2::model::Instance>,                  // 2
    internet_gateways: Vec<ec2::model::InternetGateway>,   // 3
    route_tables: Vec<ec2::model::RouteTable>,             // 4
    network_acls: Vec<ec2::model::NetworkAcl>,             // 5
    vpc_peerings: Vec<ec2::model::VpcPeeringConnection>,   // 6
    vpc_endpoints: Vec<ec2::model::VpcEndpoint>,           // 7
    nat_gateways: Vec<ec2::model::NatGateway>,             // 8
    security_groups: Vec<ec2::model::SecurityGroup>,       // 9
    vpn_connections: Vec<ec2::model::VpnConnection>,       // 10
    vpn_gateways: Vec<ec2::model::VpnGateway>,             // 11
    network_interfaces: Vec<ec2::model::NetworkInterface>, // 12
}

impl Ec2Resources {
    pub(crate) fn new(client: ec2::Client) -> Self {
        Self {
            client,
            vpcs: vec![],
            subnets: vec![],
            instances: vec![],
            internet_gateways: vec![],
            route_tables: vec![],
            network_acls: vec![],
            vpc_peerings: vec![],
            vpc_endpoints: vec![],
            nat_gateways: vec![],
            security_groups: vec![],
            vpn_connections: vec![],
            vpn_gateways: vec![],
            network_interfaces: vec![],
        }
    }

    pub(crate) async fn collect(
        &mut self,
        progress: &indicatif::ProgressBar,
    ) -> Result<(), ec2::Error> {
        progress.set_length(12);

        macro_rules! collect {
            ($collector:ident, $title:expr) => {{
                progress.set_message($title);
                self.$collector().await?;
                progress.inc(1);
            }};
        }
        collect!(collect_subnets, "Subnets");
        collect!(collect_instances, "Instances");
        collect!(collect_internet_gateways, "Internet Gateways");
        collect!(collect_route_tables, "Route Tables");
        collect!(collect_network_acls, "Network ACLs");
        collect!(collect_vpc_peerings, "VPC Peerings");
        collect!(collect_vpc_endpoints, "VPC Endpoints");
        collect!(collect_nat_gateways, "NAT Gateways");
        collect!(collect_security_groups, "Security Groups");
        collect!(collect_vpn_connections, "VPN Connections");
        collect!(collect_vpn_gateways, "VPN Gateways");
        collect!(collect_network_interfaces, "Network Interfaces");

        Ok(())
    }

    pub(crate) fn vpcs(&self) -> &[ec2::model::Vpc] {
        &self.vpcs
    }

    pub(crate) fn subnets(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::Subnet> {
        let vpc_id = Some(vpc_id.as_ref());
        self.subnets
            .iter()
            .filter(|subnet| subnet.vpc_id() == vpc_id)
            .collect()
    }

    pub(crate) fn instances(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::Instance> {
        let vpc_id = Some(vpc_id.as_ref());
        self.instances
            .iter()
            .filter(|instance| instance.vpc_id() == vpc_id)
            .collect()
    }

    pub(crate) fn internet_gateways(
        &self,
        vpc_id: impl AsRef<str>,
    ) -> Vec<&ec2::model::InternetGateway> {
        let vpc_id = Some(vpc_id.as_ref());
        self.internet_gateways
            .iter()
            .filter(|igw| {
                igw.attachments()
                    .unwrap_or_default()
                    .iter()
                    .any(|attachment| attachment.vpc_id() == vpc_id)
            })
            .collect()
    }

    pub(crate) fn route_tables(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::RouteTable> {
        let vpc_id = Some(vpc_id.as_ref());
        self.route_tables
            .iter()
            .filter(|rt| rt.vpc_id() == vpc_id)
            .collect()
    }

    pub(crate) fn network_acls(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::NetworkAcl> {
        let vpc_id = Some(vpc_id.as_ref());
        self.network_acls
            .iter()
            .filter(|nacl| nacl.vpc_id() == vpc_id)
            .collect()
    }

    pub(crate) fn vpc_peerings(
        &self,
        vpc_id: impl AsRef<str>,
    ) -> Vec<&ec2::model::VpcPeeringConnection> {
        let vpc_id = Some(vpc_id.as_ref());
        self.vpc_peerings
            .iter()
            .filter(|vpc_peering| {
                vpc_peering
                    .requester_vpc_info()
                    .map(|info| info.vpc_id() == vpc_id)
                    .unwrap_or_default()
            })
            .collect()
    }

    pub(crate) fn vpc_endpoints(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::VpcEndpoint> {
        let vpc_id = Some(vpc_id.as_ref());
        self.vpc_endpoints
            .iter()
            .filter(|vpc_endpoint| vpc_endpoint.vpc_id() == vpc_id)
            .collect()
    }

    pub(crate) fn nat_gateways(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::NatGateway> {
        let vpc_id = Some(vpc_id.as_ref());
        self.nat_gateways
            .iter()
            .filter(|nat_gateway| nat_gateway.vpc_id() == vpc_id)
            .collect()
    }

    pub(crate) fn security_groups(
        &self,
        vpc_id: impl AsRef<str>,
    ) -> Vec<&ec2::model::SecurityGroup> {
        let vpc_id = Some(vpc_id.as_ref());
        self.security_groups
            .iter()
            .filter(|security_group| security_group.vpc_id() == vpc_id)
            .collect()
    }

    pub(crate) fn vpn_connections(
        &self,
        _vpc_id: impl AsRef<str>,
    ) -> Vec<&ec2::model::VpnConnection> {
        // let vpc_id = Some(vpc_id.as_ref());
        self.vpn_connections
            .iter()
            // .filter(|vpn_connection| vpn_connection.vpc_id() == Some(vpc_id))
            .collect()
    }

    pub(crate) fn vpn_gateways(&self, _vpc_id: impl AsRef<str>) -> Vec<&ec2::model::VpnGateway> {
        // let vpc_id = Some(vpc_id.as_ref());
        self.vpn_gateways
            .iter()
            // .filter(|vpn_gateway| vpn_gateway.vpc_id() == Some(vpc_id))
            .collect()
    }

    pub(crate) fn network_interfaces(
        &self,
        vpc_id: impl AsRef<str>,
    ) -> Vec<&ec2::model::NetworkInterface> {
        let vpc_id = Some(vpc_id.as_ref());
        self.network_interfaces
            .iter()
            .filter(|network_interface| network_interface.vpc_id() == vpc_id)
            .collect()
    }

    pub(crate) async fn collect_vpcs(&mut self, vpcs: &[String]) -> Result<(), ec2::Error> {
        self.vpcs = vpcs
            .iter()
            .map(|vpc_id| ec2::model::Vpc::builder().vpc_id(vpc_id).build())
            .collect();

        let mut vpcs = vec![];
        let mut next_token = None;
        loop {
            let output = self
                .client
                .describe_vpcs()
                .set_next_token(next_token)
                .optionally_filter(self.vpc_filter())
                .send()
                .await?;
            vpcs.extend(output.vpcs.unwrap_or_default());
            next_token = output.next_token;
            if next_token.is_none() {
                break;
            }
        }

        self.vpcs = vpcs;

        // let describe_vpc = self.client.describe_vpcs();
        // let output = describe_vpcs.clone().send().await?;
        // let mut vpcs = output.vpcs.unwrap_or_default();
        // let mut next_token = output.next_token;

        // while let Some(token) = next_token {
        //     output = client.describe_vpcs().next_token(token).send().await?;
        //     vpcs.extend(output.vpcs.unwrap_or_default());
        //     next_token = output.next_token;
        // }

        Ok(())
    }

    async fn collect_subnets(&mut self) -> Result<(), ec2::Error> {
        let mut next_token = None;
        loop {
            let output = self
                .client
                .describe_subnets()
                .set_next_token(next_token)
                .optionally_filter(self.vpc_filter())
                .send()
                .await?;
            self.subnets.extend(output.subnets.unwrap_or_default());
            next_token = output.next_token;
            if next_token.is_none() {
                break;
            }
        }

        Ok(())
    }

    async fn collect_instances(&mut self) -> Result<(), ec2::Error> {
        let mut next_token = None;
        loop {
            let output = self
                .client
                .describe_instances()
                .set_next_token(next_token)
                .optionally_filter(self.vpc_filter())
                .send()
                .await?;

            let instances = output
                .reservations
                .unwrap_or_default()
                .into_iter()
                .flat_map(|reservation| reservation.instances.unwrap_or_default());

            self.instances.extend(instances);
            next_token = output.next_token;
            if next_token.is_none() {
                break;
            }
        }

        Ok(())
    }

    async fn collect_internet_gateways(&mut self) -> Result<(), ec2::Error> {
        let mut next_token = None;
        loop {
            let output = self
                .client
                .describe_internet_gateways()
                .set_next_token(next_token)
                .optionally_filter(self.attachment_vpc_filter())
                .send()
                .await?;
            self.internet_gateways
                .extend(output.internet_gateways.unwrap_or_default());
            next_token = output.next_token;
            if next_token.is_none() {
                break;
            }
        }

        Ok(())
    }

    async fn collect_route_tables(&mut self) -> Result<(), ec2::Error> {
        let mut next_token = None;
        loop {
            let output = self
                .client
                .describe_route_tables()
                .set_next_token(next_token)
                .optionally_filter(self.vpc_filter())
                .send()
                .await?;
            self.route_tables
                .extend(output.route_tables.unwrap_or_default());
            next_token = output.next_token;
            if next_token.is_none() {
                break;
            }
        }

        Ok(())
    }

    pub(super) async fn collect_network_acls(&mut self) -> Result<(), ec2::Error> {
        let mut next_token = None;
        loop {
            let output = self
                .client
                .describe_network_acls()
                .set_next_token(next_token)
                .optionally_filter(self.vpc_filter())
                .send()
                .await?;
            self.network_acls
                .extend(output.network_acls.unwrap_or_default());
            next_token = output.next_token;
            if next_token.is_none() {
                break;
            }
        }

        Ok(())
    }

    async fn collect_vpc_peerings(&mut self) -> Result<(), ec2::Error> {
        let mut next_token = None;
        loop {
            let output = self
                .client
                .describe_vpc_peering_connections()
                .set_next_token(next_token)
                .optionally_filter(self.requester_vpc_filter())
                .send()
                .await?;
            self.vpc_peerings
                .extend(output.vpc_peering_connections.unwrap_or_default());
            next_token = output.next_token;
            if next_token.is_none() {
                break;
            }
        }

        Ok(())
    }

    async fn collect_vpc_endpoints(&mut self) -> Result<(), ec2::Error> {
        let mut next_token = None;
        loop {
            let output = self
                .client
                .describe_vpc_endpoints()
                .set_next_token(next_token)
                .optionally_filter(self.vpc_filter())
                .send()
                .await?;
            self.vpc_endpoints
                .extend(output.vpc_endpoints.unwrap_or_default());
            next_token = output.next_token;
            if next_token.is_none() {
                break;
            }
        }

        Ok(())
    }

    async fn collect_nat_gateways(&mut self) -> Result<(), ec2::Error> {
        let mut next_token = None;
        loop {
            let output = self
                .client
                .describe_nat_gateways()
                .set_next_token(next_token)
                .optionally_filter(self.vpc_filter())
                .send()
                .await?;
            self.nat_gateways
                .extend(output.nat_gateways.unwrap_or_default());
            next_token = output.next_token;
            if next_token.is_none() {
                break;
            }
        }

        Ok(())
    }

    async fn collect_security_groups(&mut self) -> Result<(), ec2::Error> {
        let mut next_token = None;
        loop {
            let output = self
                .client
                .describe_security_groups()
                .set_next_token(next_token)
                .optionally_filter(self.vpc_filter())
                .send()
                .await?;
            self.security_groups
                .extend(output.security_groups.unwrap_or_default());
            next_token = output.next_token;
            if next_token.is_none() {
                break;
            }
        }

        Ok(())
    }

    // describe_vpn_connections does not support looping with next token
    async fn collect_vpn_connections(&mut self) -> Result<(), ec2::Error> {
        let vpn_connections = self
            .client
            .describe_vpn_connections()
            .optionally_filter(self.vpc_filter())
            .send()
            .await?
            .vpn_connections
            .unwrap_or_default();
        self.vpn_connections.extend(vpn_connections);

        // let mut next_token = None;
        // loop {
        //     let output = self
        //         .client
        //         .describe_vpn_connections()
        //         .set_next_token(next_token)
        //         .optionally_filter(self.vpc_filter())
        //         .send()
        //         .await?;
        //     self.vpn_connections
        //         .extend(output.vpn_connections.unwrap_or_default());
        //     next_token = output.next_token;
        //     if next_token.is_none() {
        //         break;
        //     }
        // }

        Ok(())
    }

    // describe_vpn_connections does not support looping with next token
    async fn collect_vpn_gateways(&mut self) -> Result<(), ec2::Error> {
        let vpn_gateways = self
            .client
            .describe_vpn_gateways()
            .optionally_filter(self.attachment_vpc_filter())
            .send()
            .await?
            .vpn_gateways
            .unwrap_or_default();
        self.vpn_gateways.extend(vpn_gateways);

        // let mut next_token = None;
        // loop {
        //     let output = self
        //         .client
        //         .describe_vpn_gateways()
        //         .set_next_token(next_token)
        //         .optionally_filter(self.vpc_filter())
        //         .send()
        //         .await?;
        //     self.vpn_gateways
        //         .extend(output.vpn_gateways.unwrap_or_default());
        //     next_token = output.next_token;
        //     if next_token.is_none() {
        //         break;
        //     }
        // }

        Ok(())
    }

    async fn collect_network_interfaces(&mut self) -> Result<(), ec2::Error> {
        let mut next_token = None;
        loop {
            let output = self
                .client
                .describe_network_interfaces()
                .set_next_token(next_token)
                .optionally_filter(self.vpc_filter())
                .send()
                .await?;
            self.network_interfaces
                .extend(output.network_interfaces.unwrap_or_default());
            next_token = output.next_token;
            if next_token.is_none() {
                break;
            }
        }

        Ok(())
    }

    fn vpc_filter(&self) -> Option<ec2::model::Filter> {
        let vpcs = self
            .vpcs
            .iter()
            .filter_map(|vpc| vpc.vpc_id.clone())
            .collect::<Vec<_>>();
        if vpcs.is_empty() {
            None
        } else {
            Some(filter("vpc-id", vpcs))
        }
    }

    fn attachment_vpc_filter(&self) -> Option<ec2::model::Filter> {
        let vpcs = self
            .vpcs
            .iter()
            .filter_map(|vpc| vpc.vpc_id.clone())
            .collect::<Vec<_>>();
        if vpcs.is_empty() {
            None
        } else {
            Some(filter("attachment.vpc-id", vpcs))
        }
    }

    fn requester_vpc_filter(&self) -> Option<ec2::model::Filter> {
        let vpcs = self
            .vpcs
            .iter()
            .filter_map(|vpc| vpc.vpc_id.clone())
            .collect::<Vec<_>>();
        if vpcs.is_empty() {
            None
        } else {
            Some(filter("requester-vpc-info.vpc-id", vpcs))
        }
    }
}

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

fn filter(key: &str, values: impl IntoIterator<Item = impl Into<String>>) -> ec2::model::Filter {
    let builder = ec2::model::Filter::builder().name(key);
    values
        .into_iter()
        .fold(builder, |builder, value| builder.values(value))
        .build()
}
