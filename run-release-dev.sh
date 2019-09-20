#!/bin/bash

set -e

cargo run --release purge-chain -y

cargo run --release -- --dev
