# DOB-Decoder-Standalone-Server

This project provides an one-step DOB rendering service to wrap a batch of complex steps from fetching DNA to rendering DOB traits.

Online features:
- [x] native or embeded `ckb-vm` executor
- [x] executable standalone JsonRpc server
- [x] decoder binaries temporary cache
- [x] library exported for 3rd-party integration

## `ckb-vm` executor

There are two execution modes: native and embeded.

Native mode means running a `ckb-vm` executor through invoking a pre-installed native binary, which name is highlighted [here](https://github.com/sporeprotocol/dob-decoder-standalone-server/blob/master/settings.toml#L11) in the config file. This native binary should be installed in the same machine that the decoder server runs on, and make sure it can be executed anywhere. Recommended steps to install an official VM runner:

```bash
$ git clone https://github.com/nervosnetwork/ckb-vm
$ cargo install --path . --example ckb-vm-runner
```

Embeded mode is just integrating a `ckb-vm` right in the project to execute decoder binary files, and the corresponding Cargo feature is [embeded_vm](https://github.com/sporeprotocol/dob-decoder-standalone-server/blob/master/Cargo.toml#L27) which is already marked as default. We recommend embeded mode for fresh users, in contrast, the native mode is more like an advanced usage to provide flexibility of some user-defined VM environments.

## Decoder binaries cache

Considering there would be many decoders under DOB protocol in upcoming days, caching on-chain decoders for once in cache directory, which is marked [here](https://github.com/sporeprotocol/dob-decoder-standalone-server/blob/master/settings.toml#L14), is more reasonable rather than downloading them in repeat.

Since decoder binary has two types of location according to the requirement of DOB protocol, which are respectively the `code_hash` and `type_id`, it comes up with two different persistence format to name the file, they are `code_hash_<hash>.dob` for `code_hash` and `type_id_<hash>.dob` for `type_id`. For example, decoder binary file `code_hash_edbb2d19515ebbf69be66b2178b0c4c0884fdb33878bd04a5ad68736a6af74f8.dob` means its location type is `code_hash`, with `edbb2d19515ebbf69be66b2178b0c4c0884fdb33878bd04a5ad68736a6af74f8` for the blake2b hash of its entire content.

The `code_hash` location type requires user to compile out all of interested decoder RISC-V binaries in advance, and place them into project's decoder cache directory in `code_hash_<hash>.dob` format. In contrast, the `type_id` location type has no extra demands for user to preapre, since these sort of decoder binaries have been already deployed into on-chain decoder cells which the project will automatically download from and persist into cache directory in `type_id_<hash>.dob` format.

## Launch JsonRpc server

Running a JsonRpc server allows this project to be built in case of the feature `standalone_server` opened, which is marked in [default](https://github.com/sporeprotocol/dob-decoder-standalone-server/blob/master/Cargo.toml#L27).

Steps to start the server:

```bash
$ RUST_LOG=debug cargo run
```

Command to decode DNA in on-chain Spore DOB:

```bash
$ echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "dob_decode",
    "params": [
        "<spore_id in hex format without 0x as prefix>"
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8090
```

## Protocol version

Spore DOB protocol has version identifier, like ERC721 or ERC1155, however, different versions may have totally different behaviors in decoding operation, so there is a regulation that one server instance only serves under one specific DOB version.
