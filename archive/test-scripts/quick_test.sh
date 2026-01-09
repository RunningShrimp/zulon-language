#!/bin/bash
echo "=== Quick Test of Examples ==="
echo ""

cargo run --package zulon-build --example print_call 2>&1 | tail -1
echo ""

cargo run --package zulon-build --example arc_demo 2>&1 | grep -E "Status|Complete" | tail -2
echo ""

echo "✅ Tests complete!"
