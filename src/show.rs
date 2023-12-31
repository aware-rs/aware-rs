use aws_sdk_cloudformation as cf;
use aws_sdk_ec2 as ec2;
use aws_types::region::Region;
use duplicate::duplicate_item;

pub(crate) trait Show {
    fn id(&self) -> String;

    fn tag(&self, key: &str) -> Option<&str>;

    fn description(&self) -> Option<&str> {
        None
    }

    fn name(&self) -> Option<&str> {
        self.tag("Name")
    }

    fn id_and_name(&self) -> String {
        let name = self
            .name()
            .or_else(|| self.description())
            .map(|name| format!(" ({name})"))
            .unwrap_or_default();

        format!("{}{name}", self.id())
    }
}

impl Show for Option<&Region> {
    fn id(&self) -> String {
        self.map(ToString::to_string).unwrap_or_default()
    }

    fn tag(&self, _key: &str) -> Option<&str> {
        None
    }

    fn id_and_name(&self) -> String {
        self.id()
    }
}

#[duplicate_item(
    resource id_accessor;
    [Vpc] [vpc_id];
    [InternetGateway] [internet_gateway_id];
    [Subnet] [subnet_id];
    [RouteTable] [route_table_id];
    [Instance] [instance_id];
    [NetworkAcl] [network_acl_id];
    [VpcPeeringConnection] [vpc_peering_connection_id];
    [VpcEndpoint] [vpc_endpoint_id];
    [NatGateway] [nat_gateway_id];
    [VpnConnection] [vpn_connection_id];
    [VpnGateway] [vpn_gateway_id];
)]
impl Show for &ec2::types::resource {
    fn id(&self) -> String {
        self.id_accessor().unwrap_or_default().to_string()
    }

    fn name(&self) -> Option<&str> {
        self.tag("Name")
    }

    fn tag(&self, key: &str) -> Option<&str> {
        self.tags()?
            .iter()
            .find(|tag| tag.key.as_deref() == Some(key))?
            .value()
    }
}

impl Show for &ec2::types::SecurityGroup {
    fn id(&self) -> String {
        self.group_id.clone().unwrap_or_default()
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn tag(&self, key: &str) -> Option<&str> {
        self.tags()?
            .iter()
            .find(|tag| tag.key.as_deref() == Some(key))?
            .value()
    }
}

impl Show for &ec2::types::NetworkInterface {
    fn id(&self) -> String {
        self.network_interface_id.clone().unwrap_or_default()
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn tag(&self, key: &str) -> Option<&str> {
        self.tag_set()?
            .iter()
            .find(|tag| tag.key.as_deref() == Some(key))?
            .value()
    }
}

impl Show for &cf::types::Stack {
    fn id(&self) -> String {
        self.stack_id().unwrap_or_default().to_string()
    }

    fn tag(&self, key: &str) -> Option<&str> {
        self.tags()?
            .iter()
            .find(|tag| tag.key.as_deref() == Some(key))?
            .value()
    }

    fn name(&self) -> Option<&str> {
        self.stack_name()
    }
}

impl Show for &cf::types::StackResourceSummary {
    fn id(&self) -> String {
        self.physical_resource_id().unwrap_or_default().to_string()
    }

    fn tag(&self, _key: &str) -> Option<&str> {
        None
    }

    fn name(&self) -> Option<&str> {
        self.logical_resource_id()
    }

    fn id_and_name(&self) -> String {
        let r#type = self.resource_type().unwrap_or_default();
        let resource = self.logical_resource_id().unwrap_or("unnamed resource");
        let id = self.physical_resource_id().unwrap_or_default();
        let status = self
            .resource_status()
            .map_or("no status", |status| status.as_str());
        format!("{type}: {id} ({resource}) [{status}]",)
    }
}

impl Show for cf::types::StackResourceSummary {
    fn id(&self) -> String {
        self.physical_resource_id().unwrap_or_default().to_string()
    }

    fn tag(&self, _key: &str) -> Option<&str> {
        None
    }

    fn name(&self) -> Option<&str> {
        self.logical_resource_id()
    }

    fn id_and_name(&self) -> String {
        let r#type = self.resource_type().unwrap_or_default();
        let resource = self.logical_resource_id().unwrap_or("unnamed resource");
        let id = self.physical_resource_id().unwrap_or_default();
        let status = self
            .resource_status()
            .map_or("no status", |status| status.as_str());
        format!("{type}: {id} ({resource}) [{status}]",)
    }
}

impl Show for cf::types::StackResource {
    fn id(&self) -> String {
        self.physical_resource_id().unwrap_or_default().to_string()
    }

    fn tag(&self, _key: &str) -> Option<&str> {
        None
    }

    fn name(&self) -> Option<&str> {
        self.logical_resource_id()
    }

    fn id_and_name(&self) -> String {
        let resource = self.logical_resource_id().unwrap_or("unnamed resource");
        let id = self.physical_resource_id().unwrap_or_default();
        let status = self
            .resource_status()
            .map_or("no status", |status| status.as_str());
        format!("{id} ({resource}) [{status}]",)
    }
}
