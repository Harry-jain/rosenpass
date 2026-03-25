#!/bin/bash
set -e

echo "Starting OVS Demo with Rosenpass"

# 1. Create OVS bridge manually (or rely on rosenpass auto-create)
# We will rely on Rosenpass to auto-create it, but let's clear it first if it exists
ovs-vsctl --if-exists del-br br0 || true
ovs-vsctl add-br br0

# Generate a config file with OVS enabled
cat <<EOF > config.toml
public_key = "rp-public-key"
secret_key = "rp-secret-key"
listen = ["0.0.0.0:9999"]
verbosity = "Verbose"

[network]
backend = "ovs"
bridge = "br0"
interface = "wg0"

[[peers]]
public_key = "peer-public-key"
endpoint = "127.0.0.1:10000"
key_out = "key-out.txt"
EOF

# Generate dummy keys for demo purposes
cargo run -- gen-keys --public-key rp-public-key --secret-key rp-secret-key || echo "gen-keys failed, maybe already exist"
cargo run -- gen-keys --public-key peer-public-key --secret-key peer-secret-key || echo "gen-keys failed, maybe already exist"

echo "Starting Rosenpass in background..."
cargo run -- exchange-config config.toml &
RP_PID=$!

echo "Waiting a moment for Rosenpass to initialize..."
sleep 2

echo "Showing WireGuard interface wg0 attached:"
ip link show wg0 || echo "wg0 not found"

echo "Showing OVS bridge configuration:"
ovs-vsctl show

echo "Stopping Rosenpass to demonstrate clean teardown..."
kill -TERM $RP_PID
wait $RP_PID || true

echo "Checking if interface wg0 is removed..."
ip link show wg0 && echo "wg0 still exists (ERROR)" || echo "wg0 successfully removed"

echo "Cleaning up..."
ovs-vsctl --if-exists del-br br0
rm rp-public-key rp-secret-key peer-public-key peer-secret-key config.toml key-out.txt || true

echo "Demo complete."
