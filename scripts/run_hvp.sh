#!/bin/bash

set -e
set -x

cargo build --release --no-default-features --features hvp
codesign --force --sign - \
  --entitlements entitlements.plist \
  target/release/vm-cli
./target/release/vm-cli --cpus 1 --memory 1 --accel hvp --kernel Image --cmdline "console=ttyS0,115200 earlycon=uart8250,mmio,0x09000000,115200 devtmpfs.mount=1 debug" --initramfs image.cpio.gz
