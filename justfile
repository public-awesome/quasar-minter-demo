clippy:
  cargo +nightly clippy --tests

test:
  cargo test --locked

e2e:
  go test ./e2e/...

optimize:
  if [[ $(uname -m) =~ "arm64" ]]; then \
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    --platform linux/arm64 \
    cosmwasm/workspace-optimizer-arm64:0.12.12; else \
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    --platform linux/amd64 \
    cosmwasm/workspace-optimizer:0.12.12; fi
