use std::collections::BTreeSet;

use aws_sdk_cloudformation as cf;
use tokio_stream::StreamExt;

pub(crate) use cf::types::StackStatus;

#[derive(Debug)]
pub(crate) struct CfResources {
    client: cf::Client,
    stacks: Vec<cf::types::StackSummary>,
    resources: Vec<(cf::types::StackSummary, Vec<cf::types::StackResource>)>,
}

impl CfResources {
    pub(crate) fn new(config: &aws_types::SdkConfig) -> Self {
        let client = cf::Client::new(config);

        Self {
            client,
            stacks: vec![],
            resources: vec![],
        }
    }

    pub(crate) async fn collect_stacks(
        &mut self,
        stacks: &[String],
        statuses: &[StackStatus],
    ) -> Result<(), cf::Error> {
        let requested = stacks
            .iter()
            .map(|s| s.as_str())
            .map(Some)
            .collect::<BTreeSet<_>>();
        let list_stacks = self.client.list_stacks();
        self.stacks = statuses
            .iter()
            .fold(list_stacks, |list, status| {
                list.stack_status_filter(status.clone())
            })
            .into_paginator()
            .items()
            .send()
            .filter_map(|stack| stack.ok())
            .filter(|stack| is_requested(stack, &requested))
            .collect()
            .await;

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
            let id = stack.stack_id().unwrap_or_default();
            progress.set_message(name.to_string());
            let resources = self.collect_resources(id).await?;
            self.resources.push((stack.clone(), resources));
            progress.inc(1);
        }

        Ok(())
    }

    pub(crate) fn trees(&self) -> impl Iterator<Item = termtree::Tree<String>> + '_ {
        self.resources
            .iter()
            .map(|(stack, resources)| stack_tree(stack, resources))
    }

    async fn collect_resources(
        &self,
        stack_name: &str,
    ) -> Result<Vec<cf::types::StackResource>, cf::Error> {
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
    stack: &cf::types::StackSummary,
    resources: &[cf::types::StackResource],
) -> termtree::Tree<String> {
    let mut tree = termtree::Tree::new(stack.title());
    resources.iter().for_each(|resource| {
        let r#type = resource.resource_type().unwrap_or("no type");
        let id = resource.physical_resource_id().unwrap_or("no id");
        tree.push(format!("{type:40}: {id}"));
    });

    tree
}

fn is_requested(stack: &cf::types::StackSummary, requested: &BTreeSet<Option<&str>>) -> bool {
    requested.is_empty()
        || requested.contains(&stack.stack_name())
        || requested.contains(&stack.stack_id())
}

pub(crate) fn adjust_stack_statuses(status: Vec<StackStatus>) -> Vec<StackStatus> {
    if status.is_empty() {
        // If no explicit status has been selected get everything but successfully deleted
        vec![
            StackStatus::CreateComplete,
            StackStatus::CreateFailed,
            StackStatus::CreateInProgress,
            StackStatus::DeleteFailed,
            StackStatus::DeleteInProgress,
            StackStatus::ImportComplete,
            StackStatus::ImportInProgress,
            StackStatus::ImportRollbackComplete,
            StackStatus::ImportRollbackFailed,
            StackStatus::ImportRollbackInProgress,
            StackStatus::ReviewInProgress,
            StackStatus::RollbackComplete,
            StackStatus::RollbackFailed,
            StackStatus::RollbackInProgress,
            StackStatus::UpdateComplete,
            StackStatus::UpdateCompleteCleanupInProgress,
            StackStatus::UpdateFailed,
            StackStatus::UpdateInProgress,
            StackStatus::UpdateRollbackComplete,
            StackStatus::UpdateRollbackCompleteCleanupInProgress,
            StackStatus::UpdateRollbackFailed,
            StackStatus::UpdateRollbackInProgress,
        ]
    } else if status.iter().any(|s| s.as_str().to_lowercase() == "all") {
        vec![]
    } else {
        status
    }
}

trait Title {
    fn title(&self) -> String;
}

impl Title for cf::types::Stack {
    fn title(&self) -> String {
        let name = self.stack_name().unwrap_or_default();
        if let Some(status) = self.stack_status().map(|status| status.as_str()) {
            format!("{name} ({status})")
        } else {
            name.to_string()
        }
    }
}

impl Title for cf::types::StackSummary {
    fn title(&self) -> String {
        let name = self.stack_name().unwrap_or_default();
        if let Some(status) = self.stack_status().map(|status| status.as_str()) {
            format!("{name} ({status})")
        } else {
            name.to_string()
        }
    }
}

impl Title for cf::types::StackResource {
    fn title(&self) -> String {
        let name = self.logical_resource_id().unwrap_or_default();
        if let Some(status) = self.resource_status().map(|status| status.as_str()) {
            format!("{name} ({status})")
        } else {
            name.to_string()
        }
    }
}
