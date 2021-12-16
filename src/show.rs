use aws_sdk_ec2 as ec2;

pub(crate) trait Show {
    fn id(&self) -> String;

    fn tags(&self) -> Option<&[ec2::model::Tag]>;

    fn description(&self) -> Option<&str> {
        None
    }

    fn name(&self) -> Option<&str> {
        self.tags()
            .and_then(|tags| tags.iter().find(|tag| tag.key.as_deref() == Some("Name")))
            .and_then(|tag| tag.value.as_deref())
    }

    fn id_and_name(&self) -> String {
        let name = self
            .name()
            .or_else(|| self.description())
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();

        format!("{}{}", self.id(), name)
    }
}

impl Show for Option<&ec2::Region> {
    fn id(&self) -> String {
        self.map(ToString::to_string).unwrap_or_default()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        None
    }

    fn id_and_name(&self) -> String {
        self.id()
    }
}

impl Show for ec2::model::Vpc {
    fn id(&self) -> String {
        self.vpc_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tags.as_deref()
    }
}

impl Show for &ec2::model::InternetGateway {
    fn id(&self) -> String {
        self.internet_gateway_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tags.as_deref()
    }
}

impl Show for &ec2::model::Subnet {
    fn id(&self) -> String {
        self.subnet_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tags.as_deref()
    }
}

impl Show for &ec2::model::RouteTable {
    fn id(&self) -> String {
        self.route_table_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tags.as_deref()
    }
}

impl Show for &ec2::model::Instance {
    fn id(&self) -> String {
        self.instance_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tags.as_deref()
    }
}

impl Show for &ec2::model::NetworkAcl {
    fn id(&self) -> String {
        self.network_acl_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tags.as_deref()
    }
}

impl Show for &ec2::model::VpcPeeringConnection {
    fn id(&self) -> String {
        self.vpc_peering_connection_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tags.as_deref()
    }
}

impl Show for &ec2::model::VpcEndpoint {
    fn id(&self) -> String {
        self.vpc_endpoint_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tags.as_deref()
    }
}

impl Show for &ec2::model::NatGateway {
    fn id(&self) -> String {
        self.nat_gateway_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tags.as_deref()
    }
}

impl Show for &ec2::model::SecurityGroup {
    fn id(&self) -> String {
        self.group_id.clone().unwrap_or_default()
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tags.as_deref()
    }
}

impl Show for &ec2::model::VpnConnection {
    fn id(&self) -> String {
        self.vpn_connection_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tags.as_deref()
    }
}

impl Show for &ec2::model::VpnGateway {
    fn id(&self) -> String {
        self.vpn_gateway_id.clone().unwrap_or_default()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tags.as_deref()
    }
}

impl Show for &ec2::model::NetworkInterface {
    fn id(&self) -> String {
        self.network_interface_id.clone().unwrap_or_default()
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn tags(&self) -> Option<&[ec2::model::Tag]> {
        self.tag_set.as_deref()
    }
}
