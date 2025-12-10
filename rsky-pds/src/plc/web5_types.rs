use crate::apis::ApiError;
use crate::plc::cell_data::{DidWeb5Data, DidWeb5DataUnion};
use anyhow::{bail, Result};
use ckb_jsonrpc_types::{OutPoint, Uint32};
use ckb_sdk::{Address, CkbRpcAsyncClient};
use ckb_types::{packed::Script, H256};
use molecule::prelude::Entity;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::StatusCode;
use rsky_lexicon::fans::web5::ckb::{IndexActionInputRef, PreIndexActionInputRef};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::BTreeMap, str::FromStr};
use tracing::info;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Service {
    #[serde(rename = "type")]
    pub r#type: String,
    pub endpoint: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Web5DocumentData {
    #[serde(rename = "verificationMethods")]
    pub verification_methods: BTreeMap<String, String>,
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Vec<String>,
    pub services: BTreeMap<String, Service>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateOpV1 {
    #[serde(rename = "type")]
    pub r#type: String, // string literal `create`
    #[serde(rename = "signingKey")]
    pub signing_key: String,
    #[serde(rename = "recoveryKey")]
    pub recovery_key: String,
    pub handle: String,
    pub service: String,
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Operation {
    #[serde(rename = "type")]
    pub r#type: String, // string literal `plc_operation`
    #[serde(rename = "verificationMethods")]
    pub verification_methods: BTreeMap<String, String>,
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Vec<String>,
    pub services: BTreeMap<String, Service>,
    // Omit<t.UnsignedOperation, 'prev'>
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tombstone {
    #[serde(rename = "type")]
    pub r#type: String, // string literal `plc_tombstone`
    pub prev: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)] // Needs to be signed, so we don't want an additional tag
pub enum CompatibleOpOrTombstone {
    CreateOpV1(CreateOpV1),
    Operation(Operation),
    Tombstone(Tombstone),
}

impl CompatibleOpOrTombstone {
    pub fn set_sig(&mut self, sig: String) {
        match self {
            Self::CreateOpV1(create) => create.sig = Some(sig),
            Self::Operation(op) => op.sig = Some(sig),
            Self::Tombstone(tombstone) => tombstone.sig = Some(sig),
        }
    }

    pub fn get_sig(&mut self) -> &Option<String> {
        match self {
            Self::CreateOpV1(create) => &create.sig,
            Self::Operation(op) => &op.sig,
            Self::Tombstone(tombstone) => &tombstone.sig,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)] // will be posted to API so needs to not be tagged
pub enum CompatibleOp {
    CreateOpV1(CreateOpV1),
    Operation(Operation),
}

impl CompatibleOp {
    pub fn set_sig(&mut self, sig: String) {
        match self {
            Self::CreateOpV1(create) => create.sig = Some(sig),
            Self::Operation(op) => op.sig = Some(sig),
        }
    }

    pub fn get_sig(&mut self) -> &Option<String> {
        match self {
            Self::CreateOpV1(create) => &create.sig,
            Self::Operation(op) => &op.sig,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)] // will be posted to API so needs to not be tagged
pub enum OpOrTombstone {
    Operation(Operation),
    Tombstone(Tombstone),
}

impl OpOrTombstone {
    pub fn set_sig(&mut self, sig: String) {
        match self {
            Self::Operation(op) => op.sig = Some(sig),
            Self::Tombstone(tombstone) => tombstone.sig = Some(sig),
        }
    }

    pub fn get_sig(&mut self) -> &Option<String> {
        match self {
            Self::Operation(op) => &op.sig,
            Self::Tombstone(tombstone) => &tombstone.sig,
        }
    }
}

pub async fn get_didoc_from_chain(ckb_addr: &str) -> Result<Web5DocumentData, ApiError> {
    let addr = Address::from_str(ckb_addr)
        .map_err(|_| ApiError::InvalidCkbError(format!("Address format invalid")))?;
    let script: Script = (&addr).into();
    let address_hash = "0x".to_string() + &hex::encode(script.calc_script_hash().raw_data());
    let query_url = format!("http://testnet-api.explorer.nervos.org/api/v2/scripts/referring_cells?code_hash=0x510150477b10d6ab551a509b71265f3164e9fd4137fcb5a4322f49f03092c7c5&hash_type=type&sort=created_time.asc&address_hash={}&restrict=false&page=1&page_size=1", address_hash);
    let client = reqwest::Client::new();

    let response = client
        .get(query_url)
        .send()
        .await
        .map_err(|_| ApiError::InvalidCkbError(format!("CKB Testnet no connection, retry.")))?;
    let data = response
        .text()
        .await
        .map_err(|_| ApiError::InvalidCkbError(format!("CKB Testnet Response no text.")))?;
    if data.len() == 0 {
        return Err(ApiError::CkbAddrNoCell);
    }
    let json: Value = serde_json::from_str(&data)
        .map_err(|_| ApiError::InvalidCkbError(format!("CKB Testnet Response Convert failed.")))?;

    let referring_cells = json["data"]
        .as_object()
        .ok_or(ApiError::InvalidCkbError(format!(
            "CKB Testnet Response Convert no found data"
        )))?["referring_cells"]
        .as_array()
        .ok_or(ApiError::InvalidCkbError(format!(
            "CKB Testnet Response Convert referring_cells format error."
        )))?;

    if referring_cells.len() != 0 {
        let tx_hash_str = referring_cells[0]["tx_hash"].as_str().ok_or({
            ApiError::InvalidCkbError(format!("CKB Testnet Response Convert no found tx_hash."))
        })?;
        let cell_index = referring_cells[0]["cell_index"].as_u64().ok_or({
            ApiError::InvalidCkbError(format!("CKB Testnet Response Convert no found cell_index"))
        })? as u32;

        let client = CkbRpcAsyncClient::new("https://testnet.ckb.dev/");
        let tx_hash = H256::from_str(&tx_hash_str[2..]).map_err(|_| {
            ApiError::InvalidCkbError(format!("CKB Testnet Response Convert Hash format error."))
        })?;
        let index = Uint32::from(cell_index);
        let cell = client
            .get_live_cell(OutPoint { tx_hash, index }, true)
            .await
            .map_err(|_| {
                ApiError::InvalidCkbError(format!(
                    "CKB get_live_cell error, please refresh and retry."
                ))
            })?;
        if let Some(cell) = cell.cell {
            if let Some(cell_data) = cell.data {
                let bytes = cell_data.content.as_bytes();
                let did_data = DidWeb5Data::from_slice(bytes).map_err(|_| {
                    ApiError::InvalidCkbError(
                        "DidWeb5Data convert failed, please update cell.".to_string(),
                    )
                })?;
                let DidWeb5DataUnion::DidWeb5DataV1(did_data_v1) = did_data.to_enum();
                let did_doc = did_data_v1.document();
                Ok(
                    serde_ipld_dagcbor::from_slice(&did_doc.raw_data()).map_err(|e| {
                        ApiError::InvalidCkbError(format!(
                            "Web5DocumentData dog cbor decode failed: {e:?}, please update cell."
                        ))
                    })?,
                )
            } else {
                return Err(ApiError::InvalidCkbError(
                    "Cell data not found, please update cell.".to_string(),
                ));
            }
        } else {
            return Err(ApiError::InvalidCkbError(
                "Cell info not found, please update cell.".to_string(),
            ));
        }
    } else {
        Err(ApiError::CkbDidocCellNotFound)
    }
}

pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn extract_timestamp(message: &str) -> Result<u64> {
    for line in message.lines() {
        if line.starts_with("Timestamp: ") {
            let timestamp_str = line.trim_start_matches("Timestamp: ");
            match timestamp_str.trim().parse() {
                Ok(time) => return Ok(time),
                Err(_) => bail!("Timestamp parse error"),
            }
        }
    }
    bail!("Can't find Timestamp in message!")
}

pub fn timestamp_check(timestamp: u64) -> Result<bool> {
    let current_timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(dur) => dur.as_secs(),
        Err(_) => bail!("Can't get unix timestamp from local"),
    };

    Ok(current_timestamp.saturating_sub(timestamp) < 120) // 2 min check
}

pub fn statement_check(message: &str, index: &IndexActionInputRef) -> Result<bool> {
    for line in message.lines() {
        if line.contains(&index.statement()) {
            return Ok(true);
        }
    }
    bail!("Statement check error")
}

pub fn generate_challenge(
    domain: String,
    ckb_addr: String,
    handle: String,
    index: &PreIndexActionInputRef,
) -> Result<String> {
    let ts = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(dur) => dur.as_secs(),
        Err(_) => bail!("Can't get unix timestamp from local"),
    };
    Ok(format!(
        "Web5 Login\nDomain: {}\nAddress: {}\nHandle: {}\nTimestamp: {}\nStatement: {}",
        domain,
        ckb_addr,
        handle,
        ts,
        index.statement(),
    ))
}

pub async fn get_didoc_from_indexer(did: &str) -> Result<Web5DocumentData, ApiError> {
    let indexer_url = std::env::var("INDEXER_URL").unwrap_or("http://localhost:9533/".to_owned());
    let query_url = format!("{indexer_url}{did}");
    let client = reqwest::Client::new();

    let response = client
        .get(query_url)
        .send()
        .await
        .map_err(|err| ApiError::IndexerRequestError(err.to_string()))?;
    info!("{response:?}");
    if response.status() == StatusCode::NOT_FOUND {
        return Err(ApiError::CkbDidocCellNotFound);
    }
    let data = response.text().await.map_err(|err| {
        ApiError::InvalidCkbError(format!("Indexer Response no text: {}.", err.to_string()))
    })?;

    let mut didoc: Web5DocumentData = serde_json::from_str(&data).map_err(|err| {
        ApiError::IndexerRequestError(format!(
            "Indexer Response text convert failed: {}",
            err.to_string()
        ))
    })?;

    if didoc.also_known_as.len() == 0 {
        return Err(ApiError::IncompatibleDidDoc);
    }

    let free_lu = vec![
        "at://JLer.web5.bbs.fans",
        "at://Retric.web5.bbs.fans",
        "at://ChickenDrumstick.web5.bbs.fans",
        "at://Sensen.web5.bbs.fans",
        "at://Guangzhou.web5.bbs.fans",
        "at://Shenzhen.web5.bbs.fans",
        "at://Chongqing.web5.bbs.fans",
        "at://Hangzhou.web5.bbs.fans",
        "at://LouisCK.web5.bbs.fans",
        "at://Thinker.web5.bbs.fans",
    ];

    if free_lu.contains(&didoc.also_known_as[0].as_str()) {
        didoc.also_known_as[0] = didoc.also_known_as[0].to_lowercase();
    }

    Ok(didoc)
}

pub async fn resolve_ckb_addr(ckb_addr: &str) -> Result<Vec<String>, ApiError> {
    let indexer_url = std::env::var("INDEXER_URL").unwrap_or("http://localhost:9533/".to_owned());
    let query_url = format!("{indexer_url}resolve-ckb-addr/{ckb_addr}");
    let client = reqwest::Client::new();

    let response = client
        .get(query_url)
        .send()
        .await
        .map_err(|err| ApiError::IndexerRequestError(err.to_string()))?;
    info!("{response:?}");
    if response.status() == StatusCode::NOT_FOUND {
        return Err(ApiError::CkbDidocCellNotFound);
    }
    response.json().await.map_err(|err| {
        ApiError::InvalidCkbError(format!("Indexer Response no text: {}.", err.to_string()))
    })
}

pub fn check_ckb_address(ckb_addr: &str) -> Result<(), ApiError> {
    match Address::from_str(ckb_addr) {
        Ok(address) => {
            let ckb_net = std::env::var("CKB_NETWORK").unwrap_or("ckb".into());
            if address.network().to_str() == &ckb_net {
                Ok(())
            } else {
                Err(ApiError::InvalidCkbError(format!(
                    "ckb network not matching: pds except: {ckb_net}, user input: {}",
                    address.network()
                )))
            }
        }
        Err(e) => Err(ApiError::InvalidCkbError(format!(
            "parse ckb address failed: {e}"
        ))),
    }
}
