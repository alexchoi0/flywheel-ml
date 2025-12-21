#!/bin/bash
set -e

echo "Publishing flywheel-ml crates to crates.io..."
echo "Make sure you have run: cargo login <your-api-token>"
echo ""

# Temporarily disable vendor config
if [ -f .cargo/config.toml ]; then
    mv .cargo/config.toml .cargo/config.toml.bak
    trap "mv .cargo/config.toml.bak .cargo/config.toml" EXIT
fi

# Publish in dependency order
# Each publish needs time for crates.io to index before dependents can be published

echo "1/12 Publishing flywheel-ml-core..."
cargo publish -p flywheel-ml-core
sleep 30

echo "2/12 Publishing flywheel-ml-proto..."
cargo publish -p flywheel-ml-proto
sleep 30

echo "3/12 Publishing flywheel-ml-db..."
cargo publish -p flywheel-ml-db
sleep 30

echo "4/12 Publishing flywheel-ml-dsl..."
cargo publish -p flywheel-ml-dsl
sleep 30

echo "5/12 Publishing flywheel-ml-drift..."
cargo publish -p flywheel-ml-drift
sleep 30

echo "6/12 Publishing flywheel-ml-training..."
cargo publish -p flywheel-ml-training
sleep 30

echo "7/12 Publishing flywheel-ml-client..."
cargo publish -p flywheel-ml-client
sleep 30

echo "8/12 Publishing flywheel-ml-inference..."
cargo publish -p flywheel-ml-inference
sleep 30

echo "9/12 Publishing flywheel-ml-transform..."
cargo publish -p flywheel-ml-transform
sleep 30

echo "10/12 Publishing flywheel-ml-operator..."
cargo publish -p flywheel-ml-operator
sleep 30

echo "11/12 Publishing flywheel-ml-server..."
cargo publish -p flywheel-ml-server
sleep 30

echo "12/12 Publishing flywheel-ml..."
cargo publish -p flywheel-ml

echo ""
echo "All crates published successfully!"
echo "View at: https://crates.io/crates/flywheel-ml"
