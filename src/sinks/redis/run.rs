#![allow(unused_variables)]
use super::StreamStrategy;
use crate::{model::Event, model::EventData, pipelining::StageReceiver, utils::Utils, Error};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Serialize)]
pub struct RedisRecord {
    pub event: Event,
    pub key: String,
}

impl From<Event> for RedisRecord {
    fn from(event: Event) -> Self {
        let key = key(&event);
        RedisRecord { event, key }
    }
}

fn key(event: &Event) -> String {
    if let Some(fingerprint) = &event.fingerprint {
        fingerprint.clone()
    } else {
        let mut k = String::new();
        if let Some(x) = make_key(&event) {
            k = x
        };
        k
        //event.data.clone().to_string().to_lowercase()
    }
}

pub fn producer_loop(
    input: StageReceiver,
    utils: Arc<Utils>,
    conn: &mut redis::Connection,
    stream_strategy: StreamStrategy,
    redis_stream: String,
) -> Result<(), Error> {
    for event in input.iter() {
        utils.track_sink_progress(&event);
        let payload = RedisRecord::from(event);

        let stream = match stream_strategy {
            StreamStrategy::ByEventType => payload.event.data.clone().to_string().to_lowercase(),
            _ => redis_stream.clone(),
        };

        log::debug!(
            "Stream: {:?}, Key: {:?}, Event: {:?}",
            stream,
            payload.key,
            payload.event
        );

        let _: () = redis::cmd("XADD")
            .arg(stream)
            .arg("*")
            .arg(&[(payload.key, json!(payload.event).to_string())])
            .query(conn)?;
    }

    Ok(())
}


fn make_key(event :  &Event) -> Option<String> {
    match event.data.clone() {
        EventData::Block(_) => {
            event.context.block_number.map(|n| n.to_string())
        }

        EventData::BlockEnd(_) => {
            None
        },

        EventData::Transaction(_) => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::TransactionEnd(_) => {
            None
        },

        EventData::TxInput(tx_input_record) => {
            Some(tx_input_record.tx_id +"#"+&tx_input_record.index.to_string()).map(|n| n.to_string())
        },

        EventData::TxOutput(_) => {
            let mut key = match event.context.tx_hash.clone(){
                Some(hash) => hash,
                None => return None,
            };
            if let Some(outputindex) = event.context.output_idx {
                key = key + "#" + &outputindex.to_string()
            }

            Some(key)
        },

        EventData::OutputAsset(output_asset_record) => {
            Some(output_asset_record.policy.clone()+"."+&output_asset_record.asset).map(|n| n.to_string())
        },

        EventData::Metadata(_) => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::CIP25Asset(_) => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::Mint(_) => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::Collateral {
            tx_id,
            index,
        } => {
            Some(tx_id +"#"+&index.to_string()).map(|n| n.to_string())
        },

        EventData::NativeScript {
            ..
        } => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::PlutusScript {
            ..
        } => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::StakeRegistration {
            ..
        } => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::StakeDeregistration {
            ..
        } => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::StakeDelegation {
            credential,
            pool_hash,
        } => {
            event.context.tx_hash.clone().map(|n| n.to_string() + "|" + &pool_hash)
        },

        EventData::PoolRegistration {
            ..
        }=> {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::PoolRetirement {
            pool,
            epoch,
        } => {
            Some(pool)
        },

        EventData::GenesisKeyDelegation => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::MoveInstantaneousRewardsCert {
            ..
        } => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::RollBack {
            ..
        } => {
            event.context.block_number.map(|n| n.to_string())
        },

        _ => {
            Some(event.data.clone().to_string().to_lowercase())
        }
    }
}