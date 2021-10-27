use aws_sdk_ec2 as ec2;

pub(crate) trait Show {
    fn id_and_name(&self) -> String;
}

impl Show for Option<&ec2::Region> {
    fn id_and_name(&self) -> String {
        self.map(ToString::to_string).unwrap_or_default()
    }
}

impl Show for ec2::model::Vpc {
    fn id_and_name(&self) -> String {
        let name = get_name_tag_if_any(&self.tags)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        if let Some(ref id) = self.vpc_id {
            format!("{}{}", id, name)
        } else {
            name
        }
    }
}

impl Show for ec2::model::InternetGateway {
    fn id_and_name(&self) -> String {
        let name = get_name_tag_if_any(&self.tags)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        if let Some(ref id) = self.internet_gateway_id {
            format!("{}{}", id, name)
        } else {
            name
        }
    }
}

impl Show for ec2::model::Subnet {
    fn id_and_name(&self) -> String {
        let name = get_name_tag_if_any(&self.tags)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        if let Some(ref id) = self.subnet_id {
            format!("{}{}", id, name)
        } else {
            name
        }
    }
}

fn get_name_tag_if_any(tags: &Option<Vec<ec2::model::Tag>>) -> Option<String> {
    tags.as_deref()
        .unwrap_or_default()
        .iter()
        .find(|tag| tag.key.as_deref().unwrap_or_default() == "Name")
        .map(|tag| tag.value.clone().unwrap_or_default())
}
