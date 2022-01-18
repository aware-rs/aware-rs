use aws_sdk_cloudformation as cf;
use tokio_stream::StreamExt;

#[derive(Debug)]
pub(crate) struct CfResources {
    client: cf::Client,
    stacks: Vec<cf::model::Stack>,
    resources: Vec<(cf::model::Stack, Vec<cf::model::StackResource>)>,
}

impl CfResources {
    pub(crate) fn new(config: &aws_config::Config) -> Self {
        let client = cf::Client::new(config);

        Self {
            client,
            stacks: vec![],
            resources: vec![],
        }
    }

    pub(crate) async fn collect_stacks(&mut self, stacks: &[String]) -> Result<(), cf::Error> {
        self.stacks = self
            .client
            .describe_stacks()
            .into_paginator()
            .items()
            .send()
            .filter(|stack| stacks.is_empty() || is_requested(stack, stacks))
            .collect::<Result<_, _>>()
            .await?;

        Ok(())
    }

    pub(crate) async fn collect_stack_resources(
        &mut self,
        progress: &indicatif::ProgressBar,
    ) -> Result<(), cf::Error> {
        progress.set_length(self.stacks.len() as u64);

        for stack in self.stacks.iter() {
            let name = stack
                .stack_name()
                .or_else(|| stack.stack_id())
                .unwrap_or_default();
            progress.set_message(name.to_string());
            let resources = self.collect_resources(name).await?;
            self.resources.push((stack.clone(), resources));
            progress.inc(1);
        }

        Ok(())
    }

    // pub(crate) fn stacks(&self) -> &[cf::model::Stack] {
    //     &self.stacks
    // }

    pub(crate) fn trees(&self) -> impl Iterator<Item = ptree::item::StringItem> + '_ {
        self.resources
            .iter()
            .map(|(stack, resources)| stack_tree(stack, resources))
    }

    async fn collect_resources(
        &self,
        stack_name: &str,
    ) -> Result<Vec<cf::model::StackResource>, cf::Error> {
        let resources = self
            .client
            .describe_stack_resources()
            .stack_name(stack_name)
            .send()
            .await?
            .stack_resources
            .unwrap_or_default();
        // let resources = self
        //     .client
        //     .list_stack_resources()
        //     .stack_name(stack_name)
        //     .into_paginator()
        //     .items()
        //     .send()
        //     .collect::<Result<_, _>>()
        //     .await?;

        Ok(resources)
    }
}

fn stack_tree(
    stack: &cf::model::Stack,
    resources: &[cf::model::StackResource],
) -> ptree::item::StringItem {
    let mut tree = ptree::TreeBuilder::new(stack.title());
    add_children(&mut tree, resources);
    tree.build()
}

fn is_requested(
    stack: &Result<cf::model::Stack, cf::SdkError<cf::error::DescribeStacksError>>,
    requested: &[String],
) -> bool {
    if let Ok(stack) = stack {
        requested
            .iter()
            .map(|name| name.as_str())
            .map(Some)
            .any(|name| stack.stack_name() == name || stack.stack_id() == name)
    } else {
        false
    }
}

fn add_children(ptree: &mut ptree::TreeBuilder, resources: &[cf::model::StackResource]) {
    resources.iter().for_each(|resource| {
        ptree.begin_child(resource.title());
        let r#type = resource.resource_type().unwrap_or("no type");
        let id = resource.physical_resource_id().unwrap_or("no id");
        ptree.add_empty_child(format!("{type:40}: {id}"));
        ptree.end_child();
    })
}

trait Title {
    fn title(&self) -> String;
}

impl Title for cf::model::Stack {
    fn title(&self) -> String {
        let name = self.stack_name().unwrap_or_default();
        if let Some(status) = self.stack_status().map(|status| status.as_str()) {
            format!("{name} ({status})")
        } else {
            name.to_string()
        }
    }
}

impl Title for cf::model::StackResource {
    fn title(&self) -> String {
        let name = self.logical_resource_id().unwrap_or_default();
        if let Some(status) = self.resource_status().map(|status| status.as_str()) {
            format!("{name} ({status})")
        } else {
            name.to_string()
        }
    }
}
