#!/bin/bash

set -e

cargo build

cargo run --release -- purge-chain --dev -y
cargo run --release -- --dev
