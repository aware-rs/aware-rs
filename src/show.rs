use aws_sdk_ec2 as ec2;

pub(crate) trait Show {
    fn id(&self) -> String;
    fn id_and_name(&self) -> String;
}

impl Show for Option<&ec2::Region> {
    fn id(&self) -> String {
        self.map(ToString::to_string).unwrap_or_default()
    }

    fn id_and_name(&self) -> String {
        self.id()
    }
}

impl Show for ec2::model::Vpc {
    fn id(&self) -> String {
        self.vpc_id.clone().unwrap_or_default()
    }

    fn id_and_name(&self) -> String {
        let name = get_name_tag_if_any(&self.tags)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        format!("{}{}", self.id(), name)
    }
}

impl Show for ec2::model::InternetGateway {
    fn id(&self) -> String {
        self.internet_gateway_id.clone().unwrap_or_default()
    }

    fn id_and_name(&self) -> String {
        let name = get_name_tag_if_any(&self.tags)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        format!("{}{}", self.id(), name)
    }
}

impl Show for ec2::model::Subnet {
    fn id(&self) -> String {
        self.subnet_id.clone().unwrap_or_default()
    }

    fn id_and_name(&self) -> String {
        let name = get_name_tag_if_any(&self.tags)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        format!("{}{}", self.id(), name)
    }
}

impl Show for ec2::model::RouteTable {
    fn id(&self) -> String {
        self.route_table_id.clone().unwrap_or_default()
    }

    fn id_and_name(&self) -> String {
        let name = get_name_tag_if_any(&self.tags)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        format!("{}{}", self.id(), name)
    }
}

impl Show for ec2::model::Instance {
    fn id(&self) -> String {
        self.instance_id.clone().unwrap_or_default()
    }

    fn id_and_name(&self) -> String {
        let name = get_name_tag_if_any(&self.tags)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        format!("{}{}", self.id(), name)
    }
}

fn get_name_tag_if_any(tags: &Option<Vec<ec2::model::Tag>>) -> Option<String> {
    tags.as_deref()
        .unwrap_or_default()
        .iter()
        .find(|tag| tag.key.as_deref().unwrap_or_default() == "Name")
        .map(|tag| tag.value.clone().unwrap_or_default())
}
