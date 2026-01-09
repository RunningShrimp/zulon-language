#!/bin/bash
# Fix all example BuildConfig initializations

cd crates/zulon-build/examples

for file in print_call.rs print_all.rs print_demo.rs arc_demo.rs comprehensive_io_demo.rs getchar_demo.rs greeting_demo.rs string_utils_demo.rs; do
    if [ -f "$file" ] && grep -q "Using default opt_level" "$file"; then
        echo "Fixing: $file"
        
        # Backup
        cp "$file" "${file}.bak2"
        
        # Replace the problematic pattern with correct one
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' '/\/\/ Using default opt_level: 2 (-O2)/d' "$file"
            sed -i '' '/target: None,$/a\
        ..Default::default()  // Uses opt_level: 2 (-O2)
' "$file"
        else
            sed -i '/\/\/ Using default opt_level: 2 (-O2)/d' "$file"
            sed -i '/target: None,$/a\        ..Default::default()  // Uses opt_level: 2 (-O2)' "$file"
        fi
        
        echo "  ✅ Fixed"
    fi
done

cd ../../..
echo "✅ All examples fixed!"
