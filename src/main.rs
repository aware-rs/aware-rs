use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2::{Client, Error, Region};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    let regioned_clients = client
        .describe_regions()
        .all_regions(true)
        .send()
        .await?
        .regions
        .unwrap_or_default()
        .into_iter()
        .filter_map(|region| region.region_name)
        .map(Region::new)
        // .map(|region| RegionProviderChain::first_try(region));
        .map(RegionProviderChain::first_try);

    for region in regioned_clients {
        println!("{:#?}", region.region().await);
        match get_all_vpc_from_region(region).await {
            Ok(vpc) => println!("{:#?}", vpc),
            Err(e) => println!("{:#?}", e),
        }
    }

    Ok(())
}

async fn get_all_vpc_from_region(region: RegionProviderChain) -> Result<Vec<String>, Error> {
    let config = aws_config::from_env().region(region).load().await;
    let client = Client::new(&config);
    let vpcs = client
        .describe_vpcs()
        .send()
        .await?
        .vpcs
        .unwrap_or_default()
        .into_iter()
        .filter_map(|vpc| vpc.vpc_id)
        .collect();
    Ok(vpcs)
}
