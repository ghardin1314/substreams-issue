use crate::abi;
use crate::pb::safe::v1::{
    safe_multi_sig_transaction, ExecutionFailure, ExecutionSuccess, SafeEvent,
    SafeMultiSigTransaction,
};
use crate::pb::{
    deployment::v1::Deployment,
    safe::v1::{safe_event, SafeEvents},
};
use crate::utils::{extract_metadata, keccak256};
use common::keyer;
use common::utils::pretty_hex;

use ethabi::{Address, Token, Uint};
use hex_literal::hex;
use substreams::{prelude::*};
use substreams_ethereum::pb::eth::v2::{self as eth};
use substreams_ethereum::Event;

const SAFE_TX_TYPEHASH: [u8; 32] =
    hex!("bb8310d486368db6bd6f849402fdd73ad53d316b5a4b2644ad6efe0f941286d8");
pub const DOMAIN_SEPARATOR_TYPEHASH: [u8; 32] =
    hex!("47e79534a245952e8b16893a336b85a3d9ea9fa8c573f3d803afb92a79469218");

pub const ERC191_BYTE: &'static str = "19";
pub const ERC191_VERSION: &'static str = "01";

fn map_transactions(
    event: &abi::GnosisSafeL2::events::SafeMultiSigTransaction,
) -> SafeMultiSigTransaction {
    SafeMultiSigTransaction {
        to: pretty_hex(&event.to),
        value: event.value.to_string(),
        data: pretty_hex(&event.data),
        operation: event.operation.to_u64(),
        safe_tx_gas: event.safe_tx_gas.to_string(),
        base_gas: event.base_gas.to_string(),
        gas_price: event.gas_price.to_string(),
        gas_token: pretty_hex(&event.gas_token),
        refund_receiver: pretty_hex(&event.refund_receiver),
        signatures: pretty_hex(&event.signatures),
        additional_info: pretty_hex(&event.additional_info),
        safe_tx_hash: "".to_string(),
        nonce: 0,
        result: None,
    }
}

#[substreams::handlers::map]
pub fn map_multisig_transaction(
    blk: eth::Block,
    store: StoreGetProto<Deployment>,
) -> Result<SafeEvents, substreams::errors::Error> {
    let mut events = vec![];

    for log in blk.logs() {
        let address = pretty_hex(&log.address());
        if let Some(_) = store.get_last(&keyer::deployment_key(
            &address,
            keyer::DeploymentType::Channel,
        )) {
            if let Some(event) =
                abi::GnosisSafeL2::events::SafeMultiSigTransaction::match_and_decode(log)
            {
                let mut tx_event = map_transactions(&event);

                let (safe_tx_hash, nonce) = calculate_safe_tx_hash(
                    event,
                    &log.address(),
                    137, // TODO! add as input parameter
                );

                tx_event.safe_tx_hash = pretty_hex(&safe_tx_hash);
                tx_event.nonce = nonce;

                events.push(SafeEvent {
                    r#event: Some(safe_event::Event::SafeMultisigTransaction(tx_event)),
                    metadata: extract_metadata(&log, &blk),
                    address,
                    ordinal: log.block_index() as u64,
                })
            }

            if let Some(success_event) =
                abi::GnosisSafeL2::events::ExecutionSuccess::match_and_decode(log)
            {
                events.iter_mut().for_each(|event| {
                    if let Some(safe_event::Event::SafeMultisigTransaction(tx_event)) =
                        &mut event.r#event
                    {
                        if tx_event.safe_tx_hash == pretty_hex(&success_event.tx_hash) {
                            tx_event.r#result =
                                Some(safe_multi_sig_transaction::Result::ExecutionSuccess(
                                    ExecutionSuccess {
                                        safe_tx_hash: pretty_hex(&success_event.tx_hash),
                                        payment: success_event.payment.to_string(),
                                    },
                                ));
                        }
                    }
                });
            }

            if let Some(failure_event) =
                abi::GnosisSafeL2::events::ExecutionFailure::match_and_decode(log)
            {
                events.iter_mut().for_each(|event| {
                    if let Some(safe_event::Event::SafeMultisigTransaction(tx_event)) =
                        &mut event.r#event
                    {
                        if tx_event.safe_tx_hash == pretty_hex(&failure_event.tx_hash) {
                            tx_event.r#result =
                                Some(safe_multi_sig_transaction::Result::ExecutionFailure(
                                    ExecutionFailure {
                                        safe_tx_hash: pretty_hex(&failure_event.tx_hash),
                                        payment: failure_event.payment.to_string(),
                                    },
                                ));
                        }
                    }
                });
            }
        }
    }

    Ok(SafeEvents { events })
}

fn calculate_safe_tx_hash(
    event: abi::GnosisSafeL2::events::SafeMultiSigTransaction,
    address: &[u8],
    chain_id: u64,
) -> ([u8; 32], u64) {
    let safe_type_hash: Uint = Uint::from(SAFE_TX_TYPEHASH);
    let to: Address = Address::from_slice(&event.to);
    let value: Uint = Uint::from_big_endian(&event.value.to_signed_bytes_be());
    let data: Uint = Uint::from(keccak256(&event.data.to_vec()));
    let operation: Uint = Uint::from(event.operation.to_u64());
    let safe_tx_gas: Uint = Uint::from_big_endian(&event.safe_tx_gas.to_signed_bytes_be());
    let base_gas: Uint = Uint::from_big_endian(&event.base_gas.to_signed_bytes_be());
    let gas_price: Uint = Uint::from_big_endian(&event.gas_price.to_signed_bytes_be());
    let gas_token: Address = Address::from_slice(&event.gas_token);
    let refund_receiver: Address = Address::from_slice(&event.refund_receiver);
    let nonce: Uint = Uint::from_big_endian(&event.additional_info[0..32]);

    let hash = keccak256(ethabi::encode(&[
        Token::Uint(safe_type_hash),
        Token::Address(to),              // to
        Token::Uint(value),              // value
        Token::Uint(data),               // data
        Token::Uint(operation),          // operation
        Token::Uint(safe_tx_gas),        // safe_tx_gas
        Token::Uint(base_gas),           // base_gas
        Token::Uint(gas_price),          // gas_price
        Token::Address(gas_token),       // gas_token
        Token::Address(refund_receiver), // refund_receiver
        Token::Uint(nonce),              // base_gas                // nonce
    ]));

    let domain_hash = keccak256(ethabi::encode(&[
        Token::Uint(Uint::from(DOMAIN_SEPARATOR_TYPEHASH)),
        Token::Uint(Uint::from(chain_id)),
        Token::Address(Address::from_slice(address)),
    ]));

    let mut encoded = ethabi::encode(&[
        ethabi::Token::Uint(Uint::from(domain_hash)),
        ethabi::Token::Uint(Uint::from(hash)),
    ]);
    let erc_191_byte = u8::from_str_radix(ERC191_BYTE, 16).unwrap();
    let erc_191_version = u8::from_str_radix(ERC191_VERSION, 16).unwrap();

    encoded.insert(0, erc_191_version);
    encoded.insert(0, erc_191_byte);
    (keccak256(encoded), nonce.as_u64())
}