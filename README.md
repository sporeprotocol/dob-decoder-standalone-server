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

**Get protocol versions:**

```bash
$ curl -H 'content-type: application/json' -d '{
    "id": 1,
    "jsonrpc": "2.0",
    "method": "dob_protocol_version",
    "params": []
}' http://localhost:8090
```

**Decode a spore ID:**

```bash
$ echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "dob_decode",
    "params": [
        "4f7fb83a65dae9b95c21e55d5776a84f17bb6377681befeedb20a077ce1d8aad"
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8090
```

**Decode and extract SVG from another example spore ID on mainnet:**

```bash
$ echo '{
    "id": 3,
    "jsonrpc": "2.0",
    "method": "dob_decode_svg",
    "params": [
        "0x577bf0de0dcffe2811fa827480a700bc800c8e1e9606615b1484baeea2cba830"
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8090
```

**Extract image from a btcfs or ipfs path:**

```bash
$ echo '{
    "id": 4,
    "jsonrpc": "2.0",
    "method": "dob_extract_image_from_fsuri",
    "params": [
        "btcfs://5895004e95c8a4b80f05f5314d310067a703134515d82effc2ec6eba0dda3fc9i0",
        "base64"
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8090
```

## Docker Compose

From the repository root, use Docker with the Compose v2 plugin. Prepare the
decoder directory for each network you plan to run. With the default paths,
you can seed those directories from the decoder binaries in this checkout:

```sh
mkdir -p ./cache/mainnet/decoders ./cache/testnet/decoders
cp -n ./cache/decoders/code_hash_*.bin ./cache/mainnet/decoders/
cp -n ./cache/decoders/code_hash_*.bin ./cache/testnet/decoders/
```

For your own directories, set `DOB_MAINNET_DECODERS_DIR` and/or
`DOB_TESTNET_DECODERS_DIR` in the environment or Compose `.env` file. Use
distinct, existing directories that the container can write to. Bind mounts
expose the off-chain `code_hash_<hash>.bin` files placed there, and the server
can cache on-chain decoder downloads in the same network-specific directory.

Enable either network independently; both can also run together:

```sh
docker compose -f compose.example.yaml --profile mainnet up -d --build
docker compose -f compose.example.yaml --profile testnet up -d --build
# Or start both in one command:
docker compose -f compose.example.yaml --profile mainnet --profile testnet up -d --build
```

Mainnet listens on `127.0.0.1:8090` and testnet on `127.0.0.1:8091`. Each
container reads the corresponding TOML template as `/app/settings.toml` and has
its own named volume for rendered DOBs, separate from the decoder bind mount.
Change `ckb_rpc` in the appropriate template if you
want to use a different CKB endpoint. The example is loopback-only; configure a
reverse proxy explicitly if you need remote access.

Check that the server is ready without contacting a CKB node:

```sh
curl -fsS -H 'Content-Type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"dob_protocol_version","params":[]}' \
  http://127.0.0.1:8090
```

The Docker workflow publishes an image for Linux amd64 and arm64 on successful
pushes to this repository's `master` branch. It uses a full `sha-<commit>` tag
and does not publish or update `latest` or version tags. To run a published
image rather than building locally:

```sh
export DOB_DECODER_IMAGE="ghcr.io/sporeprotocol/dob-decoder-standalone-server:sha-<commit>"
docker compose -f compose.example.yaml --profile mainnet pull
docker compose -f compose.example.yaml --profile mainnet up -d --no-build
```

Replace `<commit>` with the full Git SHA of a successful Docker workflow run.
If the package is private, authenticate with GHCR for the pull. The repository
needs write access to the package to publish; that access can be granted in
the package's **Manage Actions access** settings.
Use `docker compose -f compose.example.yaml stop mainnet` (or `stop testnet`)
to stop just one service. Bringing both down without `--volumes` keeps the
rendered DOB caches. Host decoder directories are not removed by Compose.

## Protocol version

Spore DOB protocol has unique version identifier (like ERC721 or ERC1155), however, different versions may have totally different behaviors in decoding operation, so that we come out a regulation that one server instance only serves under one specific DOB protocol version, which is marked [here](https://github.com/sporeprotocol/dob-decoder-standalone-server/blob/master/settings.toml#L2).

## Error codes

refer to error definitions [here](https://github.com/sporeprotocol/dob-decoder-standalone-server/blob/master/src/types.rs#L13).
