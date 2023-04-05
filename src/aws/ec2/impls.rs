use ec2::operation;

use super::*;

pub(super) trait Optionally {
    fn optionally_filter(self, filter: Option<ec2::types::Filter>) -> Self;
    fn fold_filters(self, filters: Vec<ec2::types::Filter>) -> Self;
}

macro_rules! impl_optionally {
    ($builder:ty) => {
        impl Optionally for $builder {
            fn optionally_filter(self, filter: Option<ec2::types::Filter>) -> Self {
                match filter {
                    Some(filter) => self.filters(filter),
                    None => self,
                }
            }

            fn fold_filters(self, filters: Vec<ec2::types::Filter>) -> Self {
                filters
                    .into_iter()
                    .fold(self, |builder, filter| builder.filters(filter))
            }
        }
    };
}

impl_optionally!(operation::describe_vpcs::builders::DescribeVpcsFluentBuilder);
impl_optionally!(operation::describe_subnets::builders::DescribeSubnetsFluentBuilder);
impl_optionally!(operation::describe_instances::builders::DescribeInstancesFluentBuilder);
impl_optionally!(
    operation::describe_internet_gateways::builders::DescribeInternetGatewaysFluentBuilder
);
impl_optionally!(operation::describe_route_tables::builders::DescribeRouteTablesFluentBuilder);
impl_optionally!(operation::describe_network_acls::builders::DescribeNetworkAclsFluentBuilder);
impl_optionally!(operation::describe_vpc_peering_connections::builders::DescribeVpcPeeringConnectionsFluentBuilder);
impl_optionally!(operation::describe_vpc_endpoints::builders::DescribeVpcEndpointsFluentBuilder);
impl_optionally!(
    operation::describe_security_groups::builders::DescribeSecurityGroupsFluentBuilder
);
impl_optionally!(
    operation::describe_vpn_connections::builders::DescribeVpnConnectionsFluentBuilder
);
impl_optionally!(operation::describe_vpn_gateways::builders::DescribeVpnGatewaysFluentBuilder);
impl_optionally!(
    operation::describe_network_interfaces::builders::DescribeNetworkInterfacesFluentBuilder
);

// impl_optionally!(DescribeNatGateways);

impl Optionally for operation::describe_nat_gateways::builders::DescribeNatGatewaysFluentBuilder {
    fn optionally_filter(self, filter: Option<ec2::types::Filter>) -> Self {
        match filter {
            Some(filter) => self.filter(filter),
            None => self,
        }
    }
    fn fold_filters(self, filters: Vec<ec2::types::Filter>) -> Self {
        filters
            .into_iter()
            .fold(self, |builder, filter| builder.filter(filter))
    }
}
