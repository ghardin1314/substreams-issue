use crate::pb::safe::v1::SafeEvents;

/// Ordering of params must match order in substreams.yaml
#[substreams::handlers::map]
pub fn map_safe_events(
    added_owners: SafeEvents,
    approve_hash: SafeEvents,
    changed_fallback_handler: SafeEvents,
    changed_guard: SafeEvents,
    changed_threshold: SafeEvents,
    disabled_module: SafeEvents,
    enabled_module: SafeEvents,
    // module_transaction: SafeEvents,
    multisig_transaction: SafeEvents,
    removed_owner: SafeEvents,
    safe_received: SafeEvents,
    safe_setup: SafeEvents,
) -> Result<SafeEvents, substreams::errors::Error> {
    let mut events = [
        added_owners.events,
        approve_hash.events,
        changed_fallback_handler.events,
        changed_guard.events,
        changed_threshold.events,
        disabled_module.events,
        enabled_module.events,
        // module_transaction.events,
        multisig_transaction.events,
        removed_owner.events,
        safe_received.events,
        safe_setup.events,
    ]
    .concat();

    events.sort_by_key(|a| {
        if let Some(metadata) = a.metadata.as_ref() {
            metadata.block_number
        } else {
            0
        }
    });

    Ok(SafeEvents { events })
}
