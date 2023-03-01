#[rustfmt::skip]
pub mod abi;
#[rustfmt::skip]
pub mod pb;


use common::{keyer, utils::pretty_hex};
use pb::{
    common::v1::TransactionMetadata,
    factory::v1::{
        factory::Type as FactoryType, ChannelFactory, Factories,
        Factory,
    },
};

use hex_literal::hex;
use substreams::{log, prelude::*};
use substreams_ethereum::{block_view::LogView, pb::eth::v2 as eth};

const REGISTRY_CONTRACT: [u8; 20] = hex!("071402d3A809f7E8b1B1b75AFa3A500D782B757f");

#[substreams::handlers::map]
pub fn map_factories(blk: eth::Block) -> Result<Factories, substreams::errors::Error> {
    Ok(Factories {
        factories: blk
            .events::<abi::Registry::events::FactoryAdded>(&[&REGISTRY_CONTRACT])
            .map(|(event, log)| {
                log::info!("Factory added: {:?}", event);

                Factory {
                    version: event.version.to_u64(),
                    r#type: map_factory_type(&event),
                    address: pretty_hex(&event.factory),
                    ordinal: log.block_index() as u64,
                    metadata: extract_metadata(&log, &blk),
                }
            })
            .collect(),
    })
}

#[substreams::handlers::store]
fn store_factories(factories: Factories, store: StoreSetProto<Factory>) {
    for factory in factories.factories {
        store.set(
            factory.ordinal,
            keyer::factory_key(&factory.address),
            &factory,
        );
    }
}

const CHANNEL_TYPE: [u8; 32] =
    hex!("446e88dcc2f366f48c68cb0da4f16d5c3aa0bd3060e71140482c0cc6bd95d989"); // keccak256(CHANNEL)

pub fn map_factory_type(event: &abi::Registry::events::FactoryAdded) -> Option<FactoryType> {
    match event.name {
        CHANNEL_TYPE => Some(FactoryType::Channel(ChannelFactory {})),
        _ => None,
    }
}

pub fn extract_metadata(
    log: &LogView,
    block: &eth::Block,
) -> Option<TransactionMetadata> {
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
