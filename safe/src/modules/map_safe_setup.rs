use crate::abi;

use crate::pb::safe::v1::SafeOwner;
use crate::pb::{
    deployment::v1::Deployment,
    safe::v1::{safe_event, SafeEvents, SafeSetup},
};
use crate::utils::extract_event_data;
use common::utils::pretty_hex;

use substreams::prelude::*;
use substreams_ethereum::pb::eth::v2 as eth;

fn map(event: abi::GnosisSafeL2::events::SafeSetup) -> safe_event::Event {
    safe_event::Event::SafeSetup(SafeSetup {
        initiator: pretty_hex(&event.initiator),
        threshold: event.threshold.to_u64(),
        initializer: pretty_hex(&event.initializer),
        fallback_handler: pretty_hex(&event.fallback_handler),
        owners: event
            .owners
            .iter()
            .map(|o| SafeOwner {
                address: pretty_hex(o),
            })
            .collect(),
    })
}

#[substreams::handlers::map]
pub fn map_safe_setup(
    blk: eth::Block,
    store: StoreGetProto<Deployment>,
) -> Result<SafeEvents, substreams::errors::Error> {
    let events = extract_event_data(&blk, &store, map);

    Ok(SafeEvents { events })
}
