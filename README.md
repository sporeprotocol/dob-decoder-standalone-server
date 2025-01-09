# DOB-Decoder-Standalone-Server

Provide an one-step DOB rendering service to squash a batch of complex steps that from DNA fetching to DOB traits rendering.

Online features:
- [x] embeded `ckb-vm` executor
- [x] executable standalone JsonRpc server
- [x] decoder binaries temporary cache
- [x] render result temporary cache
- [x] library exported for 3rd-party integration

## `ckb-vm` executor

Embeded VM executor is integrating a standalone `ckb-vm` in project to execute decoder binary files, and the corresponding feature is `embeded_vm` which is marked in [default](https://github.com/sporeprotocol/dob-decoder-standalone-server/blob/master/Cargo.toml#L27). We recommend embeded mode for fresh users, because in contrast, the native mode is more like an advanced usage for providing flexibility for user-defined VM environments.

## Decoder binaries cache

Considering there would be plenty of decoders under DOB protocol in upcoming days, caching on-chain decoders for once in cache directory, which is marked [here](https://github.com/sporeprotocol/dob-decoder-standalone-server/blob/master/settings.toml#L14), is more reasonable rather than downloading them in repeat.

Since decoder binary has two `location` types according to the requirement from DOB protocol, which are respectively the `code_hash` (name file in `code_hash_<hash>.bin` format) and `type_id` (name file in `type_id_<hash>.bin` format). For example, decoder binary file `code_hash_edbb2d19515ebbf69be66b2178b0c4c0884fdb33878bd04a5ad68736a6af74f8.dob` indicates the location type is `code_hash`, and with `edbb2d19515ebbf69be66b2178b0c4c0884fdb33878bd04a5ad68736a6af74f8` for its blake2b hash of the entire content.

The `code_hash` location type requires user to compile out all of interested decoder RISC-V binaries in advance, and then, place them into project's decoder cache directory (in `code_hash_<hash>.bin` format). In contrast, the `type_id` location type has no extra demands, since these sort of decoder binaries have been already deployed into on-chain decoder cells which the project will automatically download from and persist into cache directory (in `type_id_<hash>.bin` format).

## Render cache

Considering the immutability of Spore and Cluster cell, the DNA string in Spore cell is immutable as well, so the rendering result of DNA is indeed immutable at the same time.

Rendering output can be stored in cache directory for shorting down server response time for the same decoding requests, which is marked [here](https://github.com/sporeprotocol/dob-decoder-standalone-server/blob/master/settings.toml#L17).

## Launch JsonRpc server

Running a JsonRpc server requires project to be built under feature `standalone_server` opened, which is marked in [default](https://github.com/sporeprotocol/dob-decoder-standalone-server/blob/master/Cargo.toml#L27).

Steps to run a server:

```bash
$ RUST_LOG=dob_decoder_server=debug cargo run
```

And then, try it out:

```bash
$ echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "dob_decode",
    "params": [
        "<spore_id in hex format without 0x prefix>"
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8090
```

## Protocol version

Spore DOB protocol has unique version identifier (like ERC721 or ERC1155), however, different versions may have totally different behaviors in decoding operation, so that we come out a regulation that one server instance only serves under one specific DOB protocol version, which is marked [here](https://github.com/sporeprotocol/dob-decoder-standalone-server/blob/master/settings.toml#L2).

## Error codes

refer to error definitions [here](https://github.com/sporeprotocol/dob-decoder-standalone-server/blob/master/src/types.rs#L13).

| error code | short definition |
| -------- | ------- |
| 1001 | DnaLengthNotMatch |
| 1002 | SporeIdLengthInvalid |
| 1003 | NativeDecoderNotFound |
| 1004 | SporeIdNotFound |
| 1005 | SporeDataUncompatible |
| 1006 | SporeDataContentTypeUncompatible |
| 1007 | DOBVersionUnexpected |
| 1008 | ClusterIdNotSet |
| 1009 | ClusterIdNotFound |
| 1010 | ClusterDataUncompatible |
| 1011 | DecoderIdNotFound |
| 1012 | DecoderOutputInvalid |
| 1013 | HexedDNAParseError |
| 1014 | HexedSporeIdParseError |
| 1015 | DecoderBinaryPathInvalid |
| 1016 | DecoderExecutionError |
| 1017 | DecoderExecutionInternalError |
| 1018 | FetchLiveCellsError |
| 1019 | FetchTransactionError |
| 1020 | NoOutputCellInTransaction |
| 1021 | DOBContentUnexpected |
| 1022 | DOBMetadataUnexpected |
| 1023 | DOBRenderCacheNotFound |
| 1024 | DOBRenderCacheModified |
| 1025 | DecoderBinaryHashInvalid |
| 1026 | DecoderBinaryNotFoundInCell |
| 1027 | JsonRpcRequestError |
| 1028 | SystemTimeError |
| 1029 | DecoderHashNotFound |
| 1030 | DecoderScriptNotFound |
| 1031 | DecoderChainIsEmpty |
