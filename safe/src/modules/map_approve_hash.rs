use crate::abi;
use crate::pb::safe::v1::ApproveHash;
use crate::pb::{
    deployment::v1::Deployment,
    safe::v1::{safe_event, SafeEvents},
};
use crate::utils::extract_event_data;
use common::utils::pretty_hex;

use substreams::prelude::*;
use substreams_ethereum::pb::eth::v2 as eth;

fn map(event: abi::GnosisSafeL2::events::ApproveHash) -> safe_event::Event {
    safe_event::Event::ApproveHash(ApproveHash {
        owner: pretty_hex(&event.owner),
        hash: pretty_hex(&event.approved_hash),
    })
}

#[substreams::handlers::map]
pub fn map_approve_hash(
    blk: eth::Block,
    store: StoreGetProto<Deployment>,
) -> Result<SafeEvents, substreams::errors::Error> {
    let events = extract_event_data(&blk, &store, map);

    Ok(SafeEvents { events })
}
