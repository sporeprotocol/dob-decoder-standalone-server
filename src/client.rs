#![allow(clippy::assigning_clones)]

use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use ckb_jsonrpc_types::{CellWithStatus, JsonBytes, OutPoint, Uint32};
use ckb_sdk::rpc::ckb_indexer::{Cell, Order, Pagination, SearchKey};
use jsonrpc_core::futures::FutureExt;
use reqwest::{Client, Url};
use serde_json::Value;

use crate::types::Error;

pub type Rpc<T> = Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'static>>;

#[allow(clippy::upper_case_acronyms)]
enum Target {
    CKB,
    Indexer,
}

macro_rules! jsonrpc {
    ($method:expr, $id:expr, $self:ident, $return:ty$(, $params:ident$(,)?)*) => {{
        let data = format!(
            r#"{{"id": {}, "jsonrpc": "2.0", "method": "{}", "params": {}}}"#,
            $self.id.load(Ordering::Relaxed),
            $method,
            serde_json::to_value(($($params,)*)).unwrap()
        );
        $self.id.fetch_add(1, Ordering::Relaxed);

        let req_json: serde_json::Value = serde_json::from_str(&data).unwrap();

        let url = match $id {
            Target::CKB => $self.ckb_uri.clone(),
            Target::Indexer => $self.indexer_uri.clone(),
        };
        let c = $self.raw.post(url).json(&req_json);
        async {
            let resp = c
                .send()
                .await
                .map_err::<Error, _>(|_| Error::JsonRpcRequestError)?;
            let output = resp
                .json::<jsonrpc_core::response::Output>()
                .await
                .map_err::<Error, _>(|_| Error::JsonRpcRequestError)?;

            match output {
                jsonrpc_core::response::Output::Success(success) => {
                    Ok(serde_json::from_value::<$return>(success.result).unwrap())
                }
                jsonrpc_core::response::Output::Failure(_) => {
                    Err(Error::JsonRpcRequestError)
                }
            }
        }
    }}
}

#[derive(Clone)]
pub struct RpcClient {
    raw: Client,
    ckb_uri: Url,
    indexer_uri: Url,
    id: Arc<AtomicU64>,
}

impl RpcClient {
    pub fn new(ckb_uri: &str, indexer_uri: &str) -> Self {
        let ckb_uri = Url::parse(ckb_uri).expect("ckb uri, e.g. \"http://127.0.0.1:8114\"");
        let indexer_uri = Url::parse(indexer_uri).expect("ckb uri, e.g. \"http://127.0.0.1:8116\"");

        RpcClient {
            raw: Client::new(),
            ckb_uri,
            indexer_uri,
            id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn get_live_cell(&self, out_point: &OutPoint, with_data: bool) -> Rpc<CellWithStatus> {
        jsonrpc!(
            "get_live_cell",
            Target::CKB,
            self,
            CellWithStatus,
            out_point,
            with_data
        )
        .boxed()
    }

    pub fn get_cells(
        &self,
        search_key: SearchKey,
        limit: u32,
        cursor: Option<JsonBytes>,
    ) -> Rpc<Pagination<Cell>> {
        let order = Order::Asc;
        let limit = Uint32::from(limit);

        jsonrpc!(
            "get_cells",
            Target::Indexer,
            self,
            Pagination<Cell>,
            search_key,
            order,
            limit,
            cursor,
        )
        .boxed()
    }
}

pub struct ImageFetchClient {
    base_url: Url,
    images_cache: VecDeque<(Url, Vec<u8>)>,
    max_cache_size: usize,
}

impl ImageFetchClient {
    pub fn new(base_url: &str, cache_size: usize) -> Self {
        let base_url = Url::parse(base_url).expect("base url, e.g. \"http://127.0.0.1");
        Self {
            base_url,
            images_cache: VecDeque::new(),
            max_cache_size: cache_size,
        }
    }

    pub async fn fetch_images(&mut self, images_uri: &[String]) -> Result<Vec<Vec<u8>>, Error> {
        let mut requests = vec![];
        for uri in images_uri {
            let (tx_hash, index) = parse_uri(uri)?;
            let url = self.base_url.join(&tx_hash).expect("image url");
            let cached_image = self.images_cache.iter().find(|(v, _)| v == &url);
            if let Some((_, image)) = cached_image {
                requests.push(async { Ok((url, true, image.clone())) }.boxed());
            } else {
                requests.push(
                    async move {
                        let image = parse_image_from_btcfs(&url, index).await?;
                        Ok((url, false, image))
                    }
                    .boxed(),
                );
            }
        }
        let mut images = vec![];
        let responses = futures::future::join_all(requests).await;
        for response in responses {
            let (url, from_cache, result) = response?;
            images.push(result.to_vec());
            if !from_cache {
                self.images_cache.push_back((url, result));
                if self.images_cache.len() > self.max_cache_size {
                    self.images_cache.pop_front();
                }
            }
        }
        Ok(images)
    }
}

fn parse_uri(uri: &str) -> Result<(String, usize), Error> {
    let header = "xxxfs://".len();
    let body = uri.chars().skip(header).collect::<String>();
    let parts: Vec<&str> = body.split('i').collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err(Error::InvalidOnchainFsuriFormat);
    }
    Ok((
        parts[0].to_string(),
        parts[1]
            .parse()
            .map_err(|_| Error::InvalidOnchainFsuriFormat)?,
    ))
}

async fn parse_image_from_btcfs(url: &Url, index: usize) -> Result<Vec<u8>, Error> {
    // parse btc transaction
    let btc_tx = reqwest::get(url.clone())
        .await
        .map_err(|_| Error::FetchFromBtcNodeError)?
        .json::<Value>()
        .await
        .map_err(|_| Error::FetchFromBtcNodeError)?;
    let vin = btc_tx
        .get("vin")
        .ok_or(Error::InvalidBtcTransactionFormat)?
        .as_array()
        .ok_or(Error::InvalidBtcTransactionFormat)?
        .get(index)
        .ok_or(Error::InvalidBtcTransactionFormat)?;
    let mut witness = vin
        .get("inner_witnessscript_asm")
        .ok_or(Error::InvalidBtcTransactionFormat)?
        .as_str()
        .ok_or(Error::InvalidBtcTransactionFormat)?
        .to_owned();

    // parse inscription body
    let mut images = vec![];
    let header = "OP_IF OP_PUSHBYTES_3 444f42 OP_PUSHBYTES_1 01 OP_PUSHBYTES_9 696d6167652f706e67 OP_0 OP_PUSHDATA2 ";
    while let (Some(start), Some(end)) = (witness.find("OP_IF"), witness.find("OP_ENDIF")) {
        let inscription = &witness[start..end + "OP_ENDIF".len()];
        if !inscription.contains(header) {
            return Err(Error::InvalidInscriptionFormat);
        }
        let hexed_image = inscription
            .replace(header, "")
            .replace(" OP_ENDIF", "")
            .replace(" OP_PUSHDATA2 ", "");
        let image =
            hex::decode(hexed_image).map_err(|_| Error::InvalidInscriptionContentHexFormat)?;
        images.push(image);
        witness = witness[end + "OP_ENDIF".len()..].to_owned();
    }
    if images.is_empty() {
        return Err(Error::EmptyInscriptionContent);
    }

    Ok(images.remove(0))
}
