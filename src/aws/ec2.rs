use std::collections::HashMap;

use aws_sdk_ec2 as ec2;
use tokio_stream::StreamExt;

use crate::Show;

use impls::Optionally;

mod impls;

#[derive(Debug)]
pub(crate) struct Ec2Resources {
    client: ec2::Client,
    tags: Vec<(String, String)>,
    tag_descriptions: Vec<ec2::model::TagDescription>,
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
    pub(crate) fn new(client: ec2::Client, tags: &[(String, String)]) -> Self {
        let tags = tags.to_vec();
        Self {
            client,
            tags,
            tag_descriptions: vec![],
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

    pub(crate) fn trees(&self) -> impl Iterator<Item = ptree::item::StringItem> + '_ {
        if self.tag_descriptions.is_empty() {
            self.vpcs()
                .iter()
                .map(|vpc| self.vpc_tree(vpc))
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            vec![self.tag_tree()].into_iter()
        }
    }

    fn tag_tree(&self) -> ptree::item::StringItem {
        let mut tags: HashMap<&str, HashMap<&str, HashMap<&str, Vec<&str>>>> = HashMap::new();

        for tag in self.tag_descriptions.iter() {
            tags.entry(tag.key().unwrap_or_default())
                .or_default()
                .entry(tag.value().unwrap_or_default())
                .or_default()
                .entry(tag.resource_type().map(|r| r.as_str()).unwrap_or_default())
                .or_default()
                .push(tag.resource_id().unwrap_or_default());
        }

        let mut tree = ptree::TreeBuilder::new(String::from("Tags"));
        let tree = &mut tree;

        for (tag, values) in tags {
            tree.begin_child(tag.to_string());
            for (value, resources) in values {
                tree.begin_child(value.to_string());
                for (resource_type, resource_ids) in resources {
                    tree.begin_child(resource_type.to_string());
                    for resource_id in resource_ids {
                        tree.add_empty_child(resource_id.to_string());
                    }
                    tree.end_child();
                }
                tree.end_child();
            }
            tree.end_child();
        }
        tree.build()
    }

    fn vpc_tree(&self, vpc: &ec2::model::Vpc) -> ptree::item::StringItem {
        let mut tree = ptree::TreeBuilder::new(vpc.id_and_name());
        let tree = &mut tree;
        let vpc_id = vpc.id();
        add_children(tree, "Subnets", self.subnets(&vpc_id));
        add_children(tree, "Instances", self.instances(&vpc_id));
        add_children(tree, "Internet Gateways", self.internet_gateways(&vpc_id));
        add_children(tree, "Route Tables", self.route_tables(&vpc_id));
        add_children(tree, "Network ACLs", self.network_acls(&vpc_id));
        add_children(tree, "VPC Peering Connections", self.vpc_peerings(&vpc_id));
        add_children(tree, "VPC Endpoints", self.vpc_endpoints(&vpc_id));
        add_children(tree, "NAT Gateways", self.nat_gateways(&vpc_id));
        add_children(tree, "Security Groups", self.security_groups(&vpc_id));
        add_children(tree, "VPN Connections", self.vpn_connections(&vpc_id));
        add_children(tree, "VPN Gateways", self.vpn_gateways(&vpc_id));
        add_children(tree, "Network Interfaces", self.network_interfaces(&vpc_id));
        tree.build()
    }

    fn vpcs(&self) -> &[ec2::model::Vpc] {
        &self.vpcs
    }

    fn subnets(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::Subnet> {
        let vpc_id = Some(vpc_id.as_ref());
        self.subnets
            .iter()
            .filter(|subnet| subnet.vpc_id() == vpc_id)
            .collect()
    }

    fn instances(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::Instance> {
        let vpc_id = Some(vpc_id.as_ref());
        self.instances
            .iter()
            .filter(|instance| instance.vpc_id() == vpc_id)
            .collect()
    }

    fn internet_gateways(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::InternetGateway> {
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

    fn route_tables(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::RouteTable> {
        let vpc_id = Some(vpc_id.as_ref());
        self.route_tables
            .iter()
            .filter(|rt| rt.vpc_id() == vpc_id)
            .collect()
    }

    fn network_acls(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::NetworkAcl> {
        let vpc_id = Some(vpc_id.as_ref());
        self.network_acls
            .iter()
            .filter(|nacl| nacl.vpc_id() == vpc_id)
            .collect()
    }

    fn vpc_peerings(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::VpcPeeringConnection> {
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

    fn vpc_endpoints(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::VpcEndpoint> {
        let vpc_id = Some(vpc_id.as_ref());
        self.vpc_endpoints
            .iter()
            .filter(|vpc_endpoint| vpc_endpoint.vpc_id() == vpc_id)
            .collect()
    }

    fn nat_gateways(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::NatGateway> {
        let vpc_id = Some(vpc_id.as_ref());
        self.nat_gateways
            .iter()
            .filter(|nat_gateway| nat_gateway.vpc_id() == vpc_id)
            .collect()
    }

    fn security_groups(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::SecurityGroup> {
        let vpc_id = Some(vpc_id.as_ref());
        self.security_groups
            .iter()
            .filter(|security_group| security_group.vpc_id() == vpc_id)
            .collect()
    }

    fn vpn_connections(&self, _vpc_id: impl AsRef<str>) -> Vec<&ec2::model::VpnConnection> {
        // let vpc_id = Some(vpc_id.as_ref());
        self.vpn_connections
            .iter()
            // .filter(|vpn_connection| vpn_connection.vpc_id() == Some(vpc_id))
            .collect()
    }

    fn vpn_gateways(&self, _vpc_id: impl AsRef<str>) -> Vec<&ec2::model::VpnGateway> {
        // let vpc_id = Some(vpc_id.as_ref());
        self.vpn_gateways
            .iter()
            // .filter(|vpn_gateway| vpn_gateway.vpc_id() == Some(vpc_id))
            .collect()
    }

    fn network_interfaces(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::model::NetworkInterface> {
        let vpc_id = Some(vpc_id.as_ref());
        self.network_interfaces
            .iter()
            .filter(|network_interface| network_interface.vpc_id() == vpc_id)
            .collect()
    }

    pub(crate) async fn collect_tags(&mut self) -> Result<(), ec2::Error> {
        self.tag_descriptions = self
            .client
            .describe_tags()
            .into_paginator()
            .items()
            .send()
            .collect::<Result<_, _>>()
            .await?;
        Ok(())
    }

    pub(crate) async fn collect_vpcs(&mut self, vpcs: &[String]) -> Result<(), ec2::Error> {
        self.vpcs = vpcs
            .iter()
            .map(|vpc_id| ec2::model::Vpc::builder().vpc_id(vpc_id).build())
            .collect();

        self.vpcs = self
            .client
            .describe_vpcs()
            .optionally_filter(self.vpc_filter())
            .fold_filters(self.tag_filter())
            .into_paginator()
            .items()
            .send()
            .collect::<Result<_, _>>()
            .await?;

        Ok(())
    }

    async fn collect_subnets(&mut self) -> Result<(), ec2::Error> {
        self.subnets = self
            .client
            .describe_subnets()
            .optionally_filter(self.vpc_filter())
            .fold_filters(self.tag_filter())
            .into_paginator()
            .items()
            .send()
            .collect::<Result<_, _>>()
            .await?;

        Ok(())
    }

    async fn collect_instances(&mut self) -> Result<(), ec2::Error> {
        self.instances = self
            .client
            .describe_instances()
            .optionally_filter(self.vpc_filter())
            .fold_filters(self.tag_filter())
            .into_paginator()
            .items()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await?
            .into_iter()
            .flat_map(|reservation| reservation.instances.unwrap_or_default())
            .collect();

        Ok(())
    }

    async fn collect_internet_gateways(&mut self) -> Result<(), ec2::Error> {
        self.internet_gateways = self
            .client
            .describe_internet_gateways()
            .optionally_filter(self.attachment_vpc_filter())
            .fold_filters(self.tag_filter())
            .into_paginator()
            .items()
            .send()
            .collect::<Result<_, _>>()
            .await?;

        Ok(())
    }

    async fn collect_route_tables(&mut self) -> Result<(), ec2::Error> {
        self.route_tables = self
            .client
            .describe_route_tables()
            .optionally_filter(self.vpc_filter())
            .fold_filters(self.tag_filter())
            .into_paginator()
            .items()
            .send()
            .collect::<Result<_, _>>()
            .await?;

        Ok(())
    }

    async fn collect_network_acls(&mut self) -> Result<(), ec2::Error> {
        self.network_acls = self
            .client
            .describe_network_acls()
            .optionally_filter(self.vpc_filter())
            .fold_filters(self.tag_filter())
            .into_paginator()
            .items()
            .send()
            .collect::<Result<_, _>>()
            .await?;

        Ok(())
    }

    async fn collect_vpc_peerings(&mut self) -> Result<(), ec2::Error> {
        self.vpc_peerings = self
            .client
            .describe_vpc_peering_connections()
            .optionally_filter(self.requester_vpc_filter())
            .fold_filters(self.tag_filter())
            .into_paginator()
            .items()
            .send()
            .collect::<Result<_, _>>()
            .await?;

        Ok(())
    }

    async fn collect_vpc_endpoints(&mut self) -> Result<(), ec2::Error> {
        self.vpc_endpoints = self
            .client
            .describe_vpc_endpoints()
            .optionally_filter(self.vpc_filter())
            .fold_filters(self.tag_filter())
            .into_paginator()
            .items()
            .send()
            .collect::<Result<_, _>>()
            .await?;

        Ok(())
    }

    async fn collect_nat_gateways(&mut self) -> Result<(), ec2::Error> {
        self.nat_gateways = self
            .client
            .describe_nat_gateways()
            .optionally_filter(self.vpc_filter())
            .fold_filters(self.tag_filter())
            .into_paginator()
            .items()
            .send()
            .collect::<Result<_, _>>()
            .await?;

        Ok(())
    }

    async fn collect_security_groups(&mut self) -> Result<(), ec2::Error> {
        self.security_groups = self
            .client
            .describe_security_groups()
            .optionally_filter(self.vpc_filter())
            .fold_filters(self.tag_filter())
            .into_paginator()
            .items()
            .send()
            .collect::<Result<_, _>>()
            .await?;

        Ok(())
    }

    async fn collect_vpn_connections(&mut self) -> Result<(), ec2::Error> {
        self.vpn_connections = self
            .client
            .describe_vpn_connections()
            .optionally_filter(self.vpc_filter())
            .fold_filters(self.tag_filter())
            .send()
            .await?
            .vpn_connections
            .unwrap_or_default();

        Ok(())
    }

    async fn collect_vpn_gateways(&mut self) -> Result<(), ec2::Error> {
        self.vpn_gateways = self
            .client
            .describe_vpn_gateways()
            .optionally_filter(self.attachment_vpc_filter())
            .fold_filters(self.tag_filter())
            .send()
            .await?
            .vpn_gateways
            .unwrap_or_default();

        Ok(())
    }

    async fn collect_network_interfaces(&mut self) -> Result<(), ec2::Error> {
        self.network_interfaces = self
            .client
            .describe_network_interfaces()
            .optionally_filter(self.vpc_filter())
            .fold_filters(self.tag_filter())
            .into_paginator()
            .items()
            .send()
            .collect::<Result<_, _>>()
            .await?;

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

    fn tag_filter(&self) -> Vec<ec2::model::Filter> {
        self.tags
            .iter()
            .map(|(key, value)| (format!("tag:{key}"), [value]))
            .map(|(key, values)| filter(key, values))
            .collect()
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

pub(crate) async fn regions(client: &ec2::Client) -> Result<Vec<ec2::model::Region>, ec2::Error> {
    let regions = client
        .describe_regions()
        .all_regions(true)
        .send()
        .await?
        .regions
        .unwrap_or_default();
    Ok(regions)
}

fn filter(
    key: impl Into<String>,
    values: impl IntoIterator<Item = impl Into<String>>,
) -> ec2::model::Filter {
    let builder = ec2::model::Filter::builder().name(key);
    values
        .into_iter()
        .fold(builder, |builder, value| builder.values(value))
        .build()
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
