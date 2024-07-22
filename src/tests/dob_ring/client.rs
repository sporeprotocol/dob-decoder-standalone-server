use ckb_jsonrpc_types::{CellData, CellInfo, CellWithStatus, JsonBytes, OutPoint};
use ckb_sdk::rpc::ckb_indexer::{Cell, Pagination, SearchKey};
use ckb_types::{
    packed,
    prelude::{Builder, Entity, Pack},
    H256,
};
use futures::FutureExt;
use spore_types::{Bytes, BytesOpt, SporeData};

use crate::client::{Rpc, RpcClient, RPC};

#[derive(Clone)]
pub struct MockRpcClient {
    raw: RpcClient,
}

impl MockRpcClient {
    pub fn new(ckb_uri: &str, indexer_uri: Option<&str>) -> Self {
        let raw = RpcClient::new(ckb_uri, indexer_uri);
        Self { raw }
    }
}

impl RPC for MockRpcClient {
    fn get_live_cell(&self, out_point: &OutPoint, _with_data: bool) -> Rpc<CellWithStatus> {
        let index: u32 = out_point.index.into();
        let cluster_id = [index as u8; 32];
        let spore_data = SporeData::new_builder()
            .cluster_id(
                BytesOpt::new_builder()
                    .set(Some(
                        Bytes::new_builder()
                            .set(cluster_id.map(Into::into).to_vec())
                            .build(),
                    ))
                    .build(),
            )
            .content(
                Bytes::new_builder()
                    .set(
                        "\"ac7b88aabbcc687474703a2f2f3132372e302e302e313a383039300000\""
                            .as_bytes()
                            .into_iter()
                            .map(|v| v.clone().into())
                            .collect(),
                    )
                    .build(),
            )
            .build();
        let next_outpoint = packed::OutPoint::new(Default::default(), index + 1);
        println!("index: {}", index);
        let args = if index < 5 {
            next_outpoint.as_slice().to_vec()
        } else {
            H256::default().as_bytes().to_vec()
        };
        let live_cell = packed::CellOutput::new_builder()
            .lock(packed::Script::new_builder().args(args.pack()).build())
            .type_(Some(packed::Script::new_builder().build()).pack())
            .build();
        async move {
            Ok(CellWithStatus {
                cell: Some(CellInfo {
                    output: live_cell.into(),
                    data: Some(CellData {
                        content: JsonBytes::from_vec(spore_data.as_slice().to_vec()),
                        hash: Default::default(),
                    }),
                }),
                status: "live".to_string(),
            })
        }
        .boxed()
    }

    fn get_cells(
        &self,
        search_key: SearchKey,
        limit: u32,
        cursor: Option<JsonBytes>,
    ) -> Rpc<Pagination<Cell>> {
        self.raw.get_cells(search_key, limit, cursor)
    }
}
