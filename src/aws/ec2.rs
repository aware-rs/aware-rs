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
    tag_descriptions: Vec<ec2::types::TagDescription>,
    vpcs: Vec<ec2::types::Vpc>,
    subnets: Vec<ec2::types::Subnet>,                      // 1
    instances: Vec<ec2::types::Instance>,                  // 2
    internet_gateways: Vec<ec2::types::InternetGateway>,   // 3
    route_tables: Vec<ec2::types::RouteTable>,             // 4
    network_acls: Vec<ec2::types::NetworkAcl>,             // 5
    vpc_peerings: Vec<ec2::types::VpcPeeringConnection>,   // 6
    vpc_endpoints: Vec<ec2::types::VpcEndpoint>,           // 7
    nat_gateways: Vec<ec2::types::NatGateway>,             // 8
    security_groups: Vec<ec2::types::SecurityGroup>,       // 9
    vpn_connections: Vec<ec2::types::VpnConnection>,       // 10
    vpn_gateways: Vec<ec2::types::VpnGateway>,             // 11
    network_interfaces: Vec<ec2::types::NetworkInterface>, // 12
}

impl Ec2Resources {
    pub(crate) fn new(config: &aws_types::SdkConfig, tags: &[(String, String)]) -> Self {
        let client = ec2::Client::new(config);
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

    pub(crate) fn trees(&self) -> impl Iterator<Item = termtree::Tree<String>> + '_ {
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

    fn tag_tree(&self) -> termtree::Tree<String> {
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

        let mut tree = termtree::Tree::new("Tags".into());

        for (tag, values) in tags {
            let mut leaf = termtree::Tree::new(tag.into());
            for (value, resources) in values {
                let mut value = termtree::Tree::new(value.into());
                for (resource_type, resource_ids) in resources {
                    let r = termtree::Tree::new(resource_type.into())
                        .with_leaves(resource_ids.into_iter().map(String::from));
                    value.push(r);
                }
                leaf.push(value);
            }
            tree.push(leaf);
        }
        tree
    }

    fn vpc_tree(&self, vpc: &ec2::types::Vpc) -> termtree::Tree<String> {
        let mut tree = termtree::Tree::new(vpc.id_and_name());
        let t = &mut tree;
        let vpc_id = vpc.id();
        child_tree(t, "Subnets", self.subnets(&vpc_id));
        child_tree(t, "Instances", self.instances(&vpc_id));
        child_tree(t, "Internet Gateways", self.internet_gateways(&vpc_id));
        child_tree(t, "Route Tables", self.route_tables(&vpc_id));
        child_tree(t, "Network ACLs", self.network_acls(&vpc_id));
        child_tree(t, "VPC Peering Connections", self.vpc_peerings(&vpc_id));
        child_tree(t, "VPC Endpoints", self.vpc_endpoints(&vpc_id));
        child_tree(t, "NAT Gateways", self.nat_gateways(&vpc_id));
        child_tree(t, "Security Groups", self.security_groups(&vpc_id));
        child_tree(t, "VPN Connections", self.vpn_connections(&vpc_id));
        child_tree(t, "VPN Gateways", self.vpn_gateways(&vpc_id));
        child_tree(t, "Network Interfaces", self.network_interfaces(&vpc_id));
        tree
    }

    fn vpcs(&self) -> &[ec2::types::Vpc] {
        &self.vpcs
    }

    fn subnets(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::types::Subnet> {
        let vpc_id = Some(vpc_id.as_ref());
        self.subnets
            .iter()
            .filter(|subnet| subnet.vpc_id() == vpc_id)
            .collect()
    }

    fn instances(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::types::Instance> {
        let vpc_id = Some(vpc_id.as_ref());
        self.instances
            .iter()
            .filter(|instance| instance.vpc_id() == vpc_id)
            .collect()
    }

    fn internet_gateways(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::types::InternetGateway> {
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

    fn route_tables(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::types::RouteTable> {
        let vpc_id = Some(vpc_id.as_ref());
        self.route_tables
            .iter()
            .filter(|rt| rt.vpc_id() == vpc_id)
            .collect()
    }

    fn network_acls(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::types::NetworkAcl> {
        let vpc_id = Some(vpc_id.as_ref());
        self.network_acls
            .iter()
            .filter(|nacl| nacl.vpc_id() == vpc_id)
            .collect()
    }

    fn vpc_peerings(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::types::VpcPeeringConnection> {
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

    fn vpc_endpoints(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::types::VpcEndpoint> {
        let vpc_id = Some(vpc_id.as_ref());
        self.vpc_endpoints
            .iter()
            .filter(|vpc_endpoint| vpc_endpoint.vpc_id() == vpc_id)
            .collect()
    }

    fn nat_gateways(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::types::NatGateway> {
        let vpc_id = Some(vpc_id.as_ref());
        self.nat_gateways
            .iter()
            .filter(|nat_gateway| nat_gateway.vpc_id() == vpc_id)
            .collect()
    }

    fn security_groups(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::types::SecurityGroup> {
        let vpc_id = Some(vpc_id.as_ref());
        self.security_groups
            .iter()
            .filter(|security_group| security_group.vpc_id() == vpc_id)
            .collect()
    }

    fn vpn_connections(&self, _vpc_id: impl AsRef<str>) -> Vec<&ec2::types::VpnConnection> {
        // let vpc_id = Some(vpc_id.as_ref());
        self.vpn_connections
            .iter()
            // .filter(|vpn_connection| vpn_connection.vpc_id() == Some(vpc_id))
            .collect()
    }

    fn vpn_gateways(&self, _vpc_id: impl AsRef<str>) -> Vec<&ec2::types::VpnGateway> {
        // let vpc_id = Some(vpc_id.as_ref());
        self.vpn_gateways
            .iter()
            // .filter(|vpn_gateway| vpn_gateway.vpc_id() == Some(vpc_id))
            .collect()
    }

    fn network_interfaces(&self, vpc_id: impl AsRef<str>) -> Vec<&ec2::types::NetworkInterface> {
        let vpc_id = Some(vpc_id.as_ref());
        self.network_interfaces
            .iter()
            .filter(|network_interface| network_interface.vpc_id() == vpc_id)
            .collect()
    }

    pub(crate) async fn collect_tags(
        &mut self,
        _progress: &indicatif::ProgressBar,
    ) -> Result<(), ec2::Error> {
        self.tag_descriptions = self
            .client
            .describe_tags()
            .into_paginator()
            .items()
            .send()
            // .map(|tag_description| {
            //     if let Ok(ref tag_description) = tag_description {
            //         if let Some(key) = tag_description.key() {
            //             progress.set_message(key.to_string())
            //         }
            //         progress.inc(1);
            //     }
            //     tag_description
            // })
            .collect::<Result<_, _>>()
            .await?;
        Ok(())
    }

    pub(crate) async fn collect_vpcs(&mut self, vpcs: &[String]) -> Result<(), ec2::Error> {
        self.vpcs = vpcs
            .iter()
            .map(|vpc_id| ec2::types::Vpc::builder().vpc_id(vpc_id).build())
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

    fn vpc_filter(&self) -> Option<ec2::types::Filter> {
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

    fn tag_filter(&self) -> Vec<ec2::types::Filter> {
        self.tags
            .iter()
            .map(|(key, value)| (format!("tag:{key}"), [value]))
            .map(|(key, values)| filter(key, values))
            .collect()
    }

    fn attachment_vpc_filter(&self) -> Option<ec2::types::Filter> {
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

    fn requester_vpc_filter(&self) -> Option<ec2::types::Filter> {
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

pub(crate) async fn get_all_regions() -> Result<Vec<String>, ec2::Error> {
    let shared_config = aws_config::load_from_env().await;

    let regions = ec2::Client::new(&shared_config)
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

fn filter(
    key: impl Into<String>,
    values: impl IntoIterator<Item = impl Into<String>>,
) -> ec2::types::Filter {
    let builder = ec2::types::Filter::builder().name(key);
    values
        .into_iter()
        .fold(builder, |builder, value| builder.values(value))
        .build()
}

fn child_tree(tree: &mut termtree::Tree<String>, title: impl ToString, resources: Vec<impl Show>) {
    if !resources.is_empty() {
        let leaves = resources.into_iter().map(|resource| resource.id_and_name());
        let leaf = termtree::Tree::new(title.to_string()).with_leaves(leaves);
        tree.push(leaf);
    }
}
