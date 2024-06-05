use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use ckb_jsonrpc_types::{CellWithStatus, JsonBytes, OutPoint, Uint32};
use ckb_sdk::rpc::ckb_indexer::{Cell, Order, Pagination, SearchKey};
use ckb_vm::Bytes;
use jsonrpc_core::futures::FutureExt;
use reqwest::{Client, Url};

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
    raw: Client,
    base_url: Url,
    images_cache: VecDeque<(Url, Bytes)>,
    max_cache_size: usize,
}

impl ImageFetchClient {
    pub fn new(base_url: &str, cache_size: usize) -> Self {
        let base_url = Url::parse(base_url).expect("base url, e.g. \"http://127.0.0.1");
        Self {
            raw: Client::new(),
            base_url,
            images_cache: VecDeque::new(),
            max_cache_size: cache_size,
        }
    }

    pub async fn fetch_images(&mut self, images_uri: &[String]) -> Result<Vec<Vec<u8>>, Error> {
        let requests = images_uri
            .iter()
            .map(|uri| self.base_url.join(uri).expect("valid url"))
            .map(|url| {
                let image = self.images_cache.iter().find(|v| v.0 == url);
                if let Some((_, image)) = image {
                    async move { Ok((url, true, image.clone())) }.boxed()
                } else {
                    let send = self.raw.get(url.clone()).send();
                    async move {
                        let bytes = send
                            .await
                            .map_err(|_| Error::JsonRpcRequestError)?
                            .bytes()
                            .await
                            .map_err(|_| Error::JsonRpcRequestError)?;
                        Ok((url, false, bytes))
                    }
                    .boxed()
                }
            })
            .collect::<Vec<_>>();
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
