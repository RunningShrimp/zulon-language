#!/bin/bash
# Update all example programs to use default optimization level

echo "=== Updating Example Programs ==="
echo ""

cd crates/zulon-build/examples

# List of example files
examples=(
    "println_demo.rs"
    "print_call.rs"
    "print_all.rs"
    "print_demo.rs"
    "arc_demo.rs"
    "comprehensive_io_demo.rs"
    "getchar_demo.rs"
    "greeting_demo.rs"
    "string_utils_demo.rs"
)

for example in "${examples[@]}"; do
    if [ -f "$example" ]; then
        echo "Updating: $example"
        
        # Check if file has explicit opt_level: 0
        if grep -q "opt_level: 0," "$example"; then
            # Backup original
            cp "$example" "${example}.bak"
            
            # Replace opt_level: 0 with comment and use Default::default()
            if [[ "$OSTYPE" == "darwin"* ]]; then
                # macOS
                sed -i '' 's/opt_level: 0,/\/\/ Using default opt_level: 2 (-O2)/' "$example"
            else
                # Linux
                sed -i 's/opt_level: 0,/\/\/ Using default opt_level: 2 (-O2)/' "$example"
            fi
            
            echo "  ✅ Updated"
        else
            echo "  ⏭️  Skipped (no explicit opt_level)"
        fi
    fi
done

echo ""
echo "=== Verification ==="
echo ""
echo "Checking for remaining explicit opt_level: 0..."
grep -n "opt_level: 0," *.rs || echo "✅ All examples updated!"

echo ""
echo "✅ Update complete!"
cd ../../..
