#!/bin/bash

function main() {
  set -xu
  set +eE

  # RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-musl
  # RUSTFLAGS='-C target-feature=+crt-static' cargo build --release
  clear
  echo "cargo fmt"
  cargo fmt
  echo "cargo build --release"
  local -i _err=0

  # cargo build --release --target x86_64-unknown-linux-musl
  cargo build --release
  _err=$?
  [[ ${_err} -gt 0 ]] && echo "clipping" && . ./xclipit.bash && exit ${_err}

  set -eE
  ls -lah target/release/mitote_v026_bike_connect_backend

  # GNU strip, just in case
  /usr/bin/strip --strip-all target/release/mitote_v026_bike_connect_backend

  # UPX pack (optional; trades some startup CPU for much smaller size)
  /usr/bin/upx --best --lzma target/release/mitote_v026_bike_connect_backend

  # 6) Audit what’s heavy
  # Use these to see where bytes go:
  # cargo bloat --release -n 20
  # twiggy top -p target/release/stepper_actix   # if using wasm tools, optional
  ls -lah target/release/mitote_v026_bike_connect_backend

  mv target/release/mitote_v026_bike_connect_backend  mitote_v026_bike_connect_backend_fedora_42__64
./mitote_v026_bike_connect_backend_fedora_42__64 serve

}

main
