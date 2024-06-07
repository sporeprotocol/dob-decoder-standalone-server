#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
#[repr(i32)]
pub enum Error {
    #[error("DNA bytes length not match the requirement in Cluster")]
    DnaLengthNotMatch = 1001,
    #[error("spore id length should equal to 32")]
    SporeIdLengthInvalid,
    #[error("natvie decoder not found")]
    NativeDecoderNotFound,
    #[error("spore id not exist on-chain")]
    SporeIdNotFound,
    #[error("uncompatible spore data")]
    SporeDataUncompatible,
    #[error("uncompatible spore data content_type")]
    SporeDataContentTypeUncompatible,
    #[error("unexpected DOB protocol version")]
    DOBVersionUnexpected,
    #[error("miss cluster id in spore data")]
    ClusterIdNotSet,
    #[error("cluster id not exist on-chain")]
    ClusterIdNotFound,
    #[error("uncompatible cluster data")]
    ClusterDataUncompatible,
    #[error("decoder id not exist on-chain")]
    DecoderIdNotFound,
    #[error("output of decoder should contain at least one line")]
    DecoderOutputInvalid,
    #[error("DNA string is not in hex format")]
    HexedDNAParseError,
    #[error("spore id string is not in hex format")]
    HexedSporeIdParseError,
    #[error("invalid decoder path to persist")]
    DecoderBinaryPathInvalid,
    #[error("encounter error while executing DNA decoding")]
    DecoderExecutionError,
    #[error("decoding program triggered an error")]
    DecoderExecutionInternalError,
    #[error("encounter error while searching live cells")]
    FetchLiveCellsError,
    #[error("encounter error while searching transaction by hash")]
    FetchTransactionError,
    #[error("not found specific output_cell in transaction")]
    NoOutputCellInTransaction,
    #[error("spore content cannot parse to DOB content")]
    DOBContentUnexpected,
    #[error("cluster description cannot parse to DOB metadata")]
    DOBMetadataUnexpected,
    #[error("DOB render cache folder not found")]
    DOBRenderCacheNotFound,
    #[error("cached DOB render result file has changed unexpectedly")]
    DOBRenderCacheModified,
    #[error("invalid deployed on-chain decoder code_hash")]
    DecoderBinaryHashInvalid,
    #[error("no binary found in cell for decoder")]
    DecoderBinaryNotFoundInCell,
    #[error("error ocurred while requesing json-rpc")]
    JsonRpcRequestError,
    #[error("BTC node responsed bad")]
    FetchFromBtcNodeError,
    #[error("BTC transaction format broken")]
    InvalidBtcTransactionFormat,
    #[error("Inscription format broken")]
    InvalidInscriptionFormat,
    #[error("Inscription content must be hex format")]
    InvalidInscriptionContentHexFormat,
    #[error("Inscription content must be filled")]
    EmptyInscriptionContent,
    #[error("fs header like 'btcfs://' and 'ckbfs://' are not contained")]
    InvalidOnchainFsuriFormat,
}
