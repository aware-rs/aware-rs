use ec2::client::fluent_builders as builders;

use super::*;

pub(super) trait Optionally {
    fn optionally_filter(self, filter: Option<ec2::model::Filter>) -> Self;
    fn fold_filters(self, filters: Vec<ec2::model::Filter>) -> Self;
}

macro_rules! impl_optionally {
    ($builder:ident) => {
        impl Optionally for builders::$builder {
            fn optionally_filter(self, filter: Option<ec2::model::Filter>) -> Self {
                match filter {
                    Some(filter) => self.filters(filter),
                    None => self,
                }
            }

            fn fold_filters(self, filters: Vec<ec2::model::Filter>) -> Self {
                filters
                    .into_iter()
                    .fold(self, |builder, filter| builder.filters(filter))
            }
        }
    };
}

impl_optionally!(DescribeVpcs);
impl_optionally!(DescribeSubnets);
impl_optionally!(DescribeInstances);
impl_optionally!(DescribeInternetGateways);
impl_optionally!(DescribeRouteTables);
impl_optionally!(DescribeNetworkAcls);
impl_optionally!(DescribeVpcPeeringConnections);
impl_optionally!(DescribeVpcEndpoints);
// impl_optionally!(DescribeNatGateways);
impl_optionally!(DescribeSecurityGroups);
impl_optionally!(DescribeVpnConnections);
impl_optionally!(DescribeVpnGateways);
impl_optionally!(DescribeNetworkInterfaces);

impl Optionally for builders::DescribeNatGateways {
    fn optionally_filter(self, filter: Option<ec2::model::Filter>) -> Self {
        match filter {
            Some(filter) => self.filter(filter),
            None => self,
        }
    }
    fn fold_filters(self, filters: Vec<ec2::model::Filter>) -> Self {
        filters
            .into_iter()
            .fold(self, |builder, filter| builder.filter(filter))
    }
}
