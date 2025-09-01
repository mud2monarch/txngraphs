# ATTN @user
# This script builds the /src/examples/benchmark.rs example
# Then runs it multiple times with different workloads
# To change RAYON_THREADS, change the value in this script.
# To change chunk size, change src/reth_source.rs
# Output is saved in results.txt then the script runs `fmt.py` to save as md.
#
# You can accumulate individual results into a single file then save that to csv
# with `md2csv.py`.
#
# It's a little bit roundabout :/

#!/bin/bash

# Build once
echo "Building..."
cargo build --release --example benchmark

# Test parameters
ROOT_ADDR="0x284F11109359a7e1306C3e447ef14D38400063FF"
TOKEN_ADDR="0x4200000000000000000000000000000000000006"
RAYON_THREADS=8

echo "Running benchmarks..." > bench/results.txt
echo "========================" >> bench/results.txt

# Very Small workload
echo "=== 10K blocks, 1 depth ===" | tee -a bench/results.txt
for i in {1..5}; do
    echo "Run $i:" | tee -a bench/results.txt
    RAYON_NUM_THREADS=$RAYON_THREADS \
    RUST_LOG=off /usr/bin/time -p ./target/release/examples/benchmark \
        --root-address "$ROOT_ADDR" \
        --token-address "$TOKEN_ADDR" \
        --max-depth 1 \
        --block-start 8610738 \
        --block-end 8620738 2>&1 | tee -a bench/results.txt
    echo "" | tee -a bench/results.txt
done

# Small workload
echo "=== 40K blocks, 1 depth ===" | tee -a bench/results.txt
for i in {1..4}; do
    echo "Run $i:" | tee -a bench/results.txt
    RAYON_NUM_THREADS=$RAYON_THREADS \
    RUST_LOG=off /usr/bin/time -p ./target/release/examples/benchmark \
        --root-address "$ROOT_ADDR" \
        --token-address "$TOKEN_ADDR" \
        --max-depth 1 \
        --block-start 8610738 \
        --block-end 8650738 2>&1 | tee -a bench/results.txt
    echo "" | tee -a bench/results.txt
done

# Medium workload, 1 depth
echo "=== 100K blocks, 1 depth ===" | tee -a bench/results.txt
for i in {1..4}; do
    echo "Run $i:" | tee -a bench/results.txt
    RAYON_NUM_THREADS=$RAYON_THREADS \
    RUST_LOG=off /usr/bin/time -p ./target/release/examples/benchmark \
        --root-address "$ROOT_ADDR" \
        --token-address "$TOKEN_ADDR" \
        --max-depth 1 \
        --block-start 8610738 \
        --block-end 8710738 2>&1 | tee -a bench/results.txt
    echo "" | tee -a bench/results.txt
done

# Large workload, 1 depth
echo "=== 200K blocks, 1 depth ===" | tee -a bench/results.txt
for i in {1..4}; do
    echo "Run $i:" | tee -a bench/results.txt
    RAYON_NUM_THREADS=$RAYON_THREADS \
    RUST_LOG=off /usr/bin/time -p ./target/release/examples/benchmark \
        --root-address "$ROOT_ADDR" \
        --token-address "$TOKEN_ADDR" \
        --max-depth 1 \
        --block-start 8610738 \
        --block-end 8810738 2>&1 | tee -a bench/results.txt
    echo "" | tee -a bench/results.txt
done

# Very large workload, 2 depth
echo "=== 200K blocks, 2 depth ===" | tee -a bench/results.txt
for i in {1..4}; do
    echo "Run $i:" | tee -a bench/results.txt
    RAYON_NUM_THREADS=$RAYON_THREADS \
    RUST_LOG=off /usr/bin/time -p ./target/release/examples/benchmark \
        --root-address "$ROOT_ADDR" \
        --token-address "$TOKEN_ADDR" \
        --max-depth 2 \
        --block-start 8610738 \
        --block-end 8810738 2>&1 | tee -a bench/results.txt
    echo "" | tee -a bench/results.txt
done

uv run bench/fmt.py > bench/results.md
