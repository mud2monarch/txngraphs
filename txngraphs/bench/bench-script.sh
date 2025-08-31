#!/bin/bash

# Build once
echo "Building..."
cargo build --release --example benchmark

# Test parameters
ROOT_ADDR="0x284F11109359a7e1306C3e447ef14D38400063FF"
TOKEN_ADDR="0x4200000000000000000000000000000000000006"

echo "Running benchmarks..." > bench/results.txt
echo "========================" >> bench/results.txt

# Small workload
echo "=== 5K blocks, 1 depth ===" | tee -a bench/results.txt
for i in {1..3}; do
    echo "Run $i:" | tee -a bench/results.txt
    RUST_LOG=off /usr/bin/time -p ./target/release/examples/benchmark \
        --root-address "$ROOT_ADDR" \
        --token-address "$TOKEN_ADDR" \
        --max-depth 1 \
        --block-start 8610738 \
        --block-end 8615738 2>&1 | tee -a bench/results.txt
    echo "" | tee -a bench/results.txt
done

# Medium workload
echo "=== 20K blocks, 1 depth ===" | tee -a bench/results.txt
for i in {1..3}; do
    echo "Run $i:" | tee -a bench/results.txt
    RUST_LOG=off /usr/bin/time -p ./target/release/examples/benchmark \
        --root-address "$ROOT_ADDR" \
        --token-address "$TOKEN_ADDR" \
        --max-depth 1 \
        --block-start 8610738 \
        --block-end 8630738 2>&1 | tee -a bench/results.txt
    echo "" | tee -a bench/results.txt
done

echo "=== 100K blocks, 1 depth ===" | tee -a bench/results.txt
for i in {1..3}; do
    echo "Run $i:" | tee -a bench/results.txt
    RUST_LOG=off /usr/bin/time -p ./target/release/examples/benchmark \
        --root-address "$ROOT_ADDR" \
        --token-address "$TOKEN_ADDR" \
        --max-depth 1 \
        --block-start 8610738 \
        --block-end 8710738 2>&1 | tee -a bench/results.txt
    echo "" | tee -a bench/results.txt
done

echo "Results saved to bench/results.txt"
