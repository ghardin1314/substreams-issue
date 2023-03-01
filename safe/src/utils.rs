use substreams_ethereum::{block_view::LogView, pb::eth::v2::Block};
use tiny_keccak::{Hasher, Keccak};

use crate::pb::common::v1::TransactionMetadata;
use crate::pb::{
    deployment::v1::Deployment,
    safe::v1::{safe_event, SafeEvent},
};
use common::{keyer, utils::pretty_hex};

use substreams::prelude::*;
use substreams_ethereum::pb::eth::v2 as eth;
use substreams_ethereum::Event;

pub fn extract_metadata(log: &LogView, block: &Block) -> Option<TransactionMetadata> {
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

/// Matches the event address to a know safe deployment and decodes the event
pub fn extract_event_data<T: Event>(
    block: &eth::Block,
    store: &StoreGetProto<Deployment>,
    map: fn(T) -> safe_event::Event,
) -> Vec<SafeEvent> {
    let mut events = vec![];

    for log in block.logs() {
        let address = pretty_hex(&log.address());
        if let Some(_) = store.get_last(&keyer::deployment_key(
            &address,
            keyer::DeploymentType::Channel,
        )) {
            if let Some(event) = T::match_and_decode(log) {
                events.push(SafeEvent {
                    r#event: Some(map(event)),
                    metadata: extract_metadata(&log, &block),
                    address,
                    ordinal: log.block_index() as u64,
                })
            }
        }
    }

    events
}

pub fn keccak256<S>(bytes: S) -> [u8; 32]
where
    S: AsRef<[u8]>,
{
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes.as_ref());
    hasher.finalize(&mut output);
    output
}
