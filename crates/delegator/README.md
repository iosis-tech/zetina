## zetina-delegator

## `zetina-submit`

Install:

```sh
cargo install  -f --path crates/delegator/
```

Command:

```console
❯ zetina-submit --help
A shared peer-to-peer network of Zero-Knowledge Provers

Usage: zetina-submit <NETWORK> <PRIVATE_KEY> <ACCOUNT_ADDRESS> <RPC_URL>

Arguments:
  <NETWORK>          [possible values: mainnet, sepolia]
  <PRIVATE_KEY>
  <ACCOUNT_ADDRESS>
  <RPC_URL>

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Example Usage:

```console
zetina-submit sepolia 07c7a41c77c7a3b19e7c77485854fc88b09ed7041361595920009f81236d55d2 cdd51fbc4e008f4ef807eaf26f5043521ef5931bbb1e04032a25bd845d286b https://starknet-sepolia.public.blastapi.io
```
