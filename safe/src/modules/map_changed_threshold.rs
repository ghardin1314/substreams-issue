use crate::abi;
use crate::pb::safe::v1::ChangedThreshold;
use crate::pb::{
    deployment::v1::Deployment,
    safe::v1::{safe_event, SafeEvents},
};
use crate::utils::extract_event_data;

use substreams::prelude::*;
use substreams_ethereum::pb::eth::v2 as eth;

fn map(event: abi::GnosisSafeL2::events::ChangedThreshold) -> safe_event::Event {
    safe_event::Event::ChangedThreshold(ChangedThreshold {
        threshold: event.threshold.to_u64(),
    })
}

#[substreams::handlers::map]
pub fn map_changed_threshold(
    blk: eth::Block,
    store: StoreGetProto<Deployment>,
) -> Result<SafeEvents, substreams::errors::Error> {
    let events = extract_event_data(&blk, &store, map);

    Ok(SafeEvents { events })
}
