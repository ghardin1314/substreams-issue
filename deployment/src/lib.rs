#[rustfmt::skip]
pub mod abi;
#[rustfmt::skip]
pub mod pb;

use common::{keyer, utils::pretty_hex};

use pb::{
    common::v1::TransactionMetadata,
    deployment::v1::{
        deployment::Type as DeploymentType, ChannelDeployment, Deployment, Deployments,
    },
    factory::v1::Factory,
};

use substreams::{log, prelude::*};
use substreams_ethereum::Event;
use substreams_ethereum::{block_view::LogView, pb::eth::v2 as eth};

#[substreams::handlers::map]
fn map_deployments(
    blk: eth::Block,
    store: StoreGetProto<Factory>,
) -> Result<Deployments, substreams::errors::Error> {
    let mut deployments = vec![];
    for log in blk.logs() {
        let factory_address = pretty_hex(&log.address());
        // Extract channel deployed events
        if let Some(event) = abi::ChannelFactory::events::ChannelDeployed::match_and_decode(log) {
            match store.get_last(keyer::factory_key(&factory_address)) {
                None => {
                    log::debug!("Channel Factory not found: {}", factory_address);
                }
                Some(factory) => {
                    log::info!("Channel deployed: {:?}", event);
                    let channel = Deployment {
                        address: pretty_hex(&event.channel),
                        r#type: Some(DeploymentType::Channel(ChannelDeployment {
                            deployer: pretty_hex(&event.deployer),
                        })),
                        ordinal: log.block_index() as u64,
                        version: factory.version,
                        metadata: extract_metadata(&log, &blk),
                    };
                    deployments.push(channel);
                }
            }
        }
    }

    Ok(Deployments { deployments })
}

#[substreams::handlers::store]
pub fn store_deployments(deployments: Deployments, store: StoreSetProto<Deployment>) {
    for deployment in deployments.deployments {
        let deployment_type = match &deployment.r#type {
            Some(DeploymentType::Channel(_)) => keyer::DeploymentType::Channel,
            Some(DeploymentType::Splits(_)) => keyer::DeploymentType::Splits,
            Some(DeploymentType::EditionsDirectDrop(_)) => {
                keyer::DeploymentType::EditionsDirectDrop
            }
            None => keyer::DeploymentType::Unknown,
        };

        store.set(
            deployment.ordinal,
            keyer::deployment_key(&deployment.address, deployment_type),
            &deployment,
        );
    }
}

pub fn extract_metadata(log: &LogView, block: &eth::Block) -> Option<TransactionMetadata> {
    Some(TransactionMetadata {
        tx_hash: pretty_hex(&log.receipt.transaction.hash),
        block_number: block.number,
        block_timestamp: block.timestamp_seconds(),
        to: pretty_hex(&log.receipt.transaction.to),
        from: pretty_hex(&log.receipt.transaction.from),
        log_index: log.log.index,
        block_index: log.log.block_index,
    })
}
