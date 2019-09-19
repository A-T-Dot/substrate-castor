#!/bin/bash

set -e

cargo build

cargo run -- purge-chain --dev -y
cargo run -- --dev