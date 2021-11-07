use aws_sdk_ec2 as ec2;

pub(crate) trait Show {
    fn id(&self) -> String;
    fn tags(&self) -> &Option<Vec<ec2::model::Tag>>;
    fn id_and_name(&self) -> String {
        format!("{}{}", self.id(), format_name(self.tags()))
    }
}

impl Show for Option<&ec2::Region> {
    fn id(&self) -> String {
        self.map(ToString::to_string).unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &None
    }

    fn id_and_name(&self) -> String {
        self.id()
    }
}

impl Show for ec2::model::Vpc {
    fn id(&self) -> String {
        self.vpc_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tags
    }
}

impl Show for ec2::model::InternetGateway {
    fn id(&self) -> String {
        self.internet_gateway_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tags
    }
}

impl Show for ec2::model::Subnet {
    fn id(&self) -> String {
        self.subnet_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tags
    }
}

impl Show for ec2::model::RouteTable {
    fn id(&self) -> String {
        self.route_table_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tags
    }
}

impl Show for ec2::model::Instance {
    fn id(&self) -> String {
        self.instance_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tags
    }
}

impl Show for ec2::model::NetworkAcl {
    fn id(&self) -> String {
        self.network_acl_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tags
    }
}

impl Show for ec2::model::VpcPeeringConnection {
    fn id(&self) -> String {
        self.vpc_peering_connection_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tags
    }
}

impl Show for ec2::model::VpcEndpoint {
    fn id(&self) -> String {
        self.vpc_endpoint_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tags
    }
}

impl Show for ec2::model::NatGateway {
    fn id(&self) -> String {
        self.nat_gateway_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tags
    }
}

impl Show for ec2::model::SecurityGroup {
    fn id(&self) -> String {
        self.group_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tags
    }
}

impl Show for ec2::model::VpnConnection {
    fn id(&self) -> String {
        self.vpn_connection_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tags
    }
}

impl Show for ec2::model::VpnGateway {
    fn id(&self) -> String {
        self.vpn_gateway_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tags
    }
}

impl Show for ec2::model::NetworkInterface {
    fn id(&self) -> String {
        self.network_interface_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> &Option<Vec<ec2::model::Tag>> {
        &self.tag_set
    }
}

fn format_name(tags: &Option<Vec<ec2::model::Tag>>) -> String {
    get_name_tag_if_any(tags)
        .map(|name| format!(" ({})", name))
        .unwrap_or_default()
}

fn get_name_tag_if_any(tags: &Option<Vec<ec2::model::Tag>>) -> Option<String> {
    tags.as_deref()
        .unwrap_or_default()
        .iter()
        .find(|tag| tag.key.as_deref().unwrap_or_default() == "Name")
        .map(|tag| tag.value.clone().unwrap_or_default())
}
