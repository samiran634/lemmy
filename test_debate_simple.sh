#!/bin/bash
# Simple test script for debate system

echo "Testing debate system compilation..."

# Test just the debate crate
echo "1. Testing lemmy_debate crate..."
cargo check --package lemmy_debate --features full 2>&1 | grep -E "(Finished|error\[E)"

# Test the database schema
echo "2. Testing database schema..."
cargo check --package lemmy_db_schema --features full 2>&1 | grep -E "(Finished|error\[E)"

# Test utils
echo "3. Testing utils..."
cargo check --package lemmy_utils --features full 2>&1 | grep -E "(Finished|error\[E)"

echo "Done!"
