#!/bin/bash
# Run the bot with .env.test configuration
set -a
source .env.test
set +a
cargo run "$@"
