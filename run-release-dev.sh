#!/bin/bash

set -e

if [ -s ./target/release/castor ]; then
  ./target/release/castor purge-chain --dev -y
else
  cargo run --release -- purge-chain --dev -y
fi

cargo run --release -- --dev
