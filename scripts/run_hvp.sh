#!/bin/bash

set -e
set -x

rm -f /tmp/vm.sock

cargo build --release
codesign --force --sign - \
  --entitlements entitlements.plist \
  target/release/vm-cli
./target/release/vm-cli json --path aarch64.json

