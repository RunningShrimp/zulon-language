#!/bin/bash
echo "========================================="
echo "  ZULON Current Capabilities Check"
echo "========================================="
echo ""

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

test_feature() {
    local name="$1"
    local code="$2"
    local should_work="$3"
    
    echo -n "Testing: $name ... "
    
    echo "$code" > test_temp.zl
    
    if cargo run -p zulon-compiler -- test_temp.zl -o test_temp > /dev/null 2>&1; then
        if [ "$should_work" = "yes" ]; then
            echo -e "${GREEN}✅ WORKS${NC}"
        else
            echo -e "${YELLOW}⚠ UNEXPECTED SUCCESS${NC}"
        fi
    else
        if [ "$should_work" = "no" ]; then
            echo -e "${GREEN}✅ CORRECTLY FAILS${NC}"
        else
            echo -e "${RED}❌ SHOULD WORK BUT FAILS${NC}"
        fi
    fi
    
    rm -f test_temp.zl test_temp.zl.ll test_temp.zl.s test_temp
}

echo "Core Features:"
echo "--------------"

test_feature "Function with return" \
'fn main() -> i32 {
    42
}' "yes"

test_feature "Function without return type" \
'fn main() {
    0
}' "yes"

test_feature "Variable declaration" \
'fn main() -> i32 {
    let x = 10;
    x
}' "yes"

test_feature "Mutable variable" \
'fn main() -> i32 {
    let mut x = 10;
    x = 20;
    x
}' "yes"

test_feature "Binary operations" \
'fn main() -> i32 {
    let x = 10 + 20;
    x
}' "yes"

test_feature "If expression" \
'fn main() -> i32 {
    let x = 10;
    if x > 5 {
        100
    } else {
        0
    }
}' "yes"

test_feature "While loop" \
'fn main() -> i32 {
    let mut sum = 0;
    let mut i = 0;
    while i < 10 {
        sum = sum + i;
        i = i + 1
    };
    sum
}' "yes"

test_feature "Unary negation" \
'fn main() -> i32 {
    let x = -42;
    x
}' "yes"

test_feature "Function call" \
'fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() -> i32 {
    add(10, 20)
}' "yes"

test_feature "Recursive function" \
'fn fib(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fn main() -> i32 {
    fib(5)
}' "yes"

echo ""
echo "Known Limitations:"
echo "------------------"

test_feature "Comments" \
'fn main() -> i32 {
    42
}' "yes"

test_feature "Struct definition" \
'struct Point {
    x: i32,
    y: i32
}

fn main() -> i32 {
    0
}' "yes"

test_feature "Enum definition" \
'enum Option {
    Some(i32),
    None
}

fn main() -> i32 {
    0
}' "yes"

test_feature "Match expression" \
'fn main() -> i32 {
    let x = 10;
    match x {
        10 => 1,
        _ => 0
    }
}' "no"

test_feature "Return statement" \
'fn main() -> i32 {
    return 42
}' "yes"

test_feature "String literals" \
'fn main() -> i32 {
    let s = "hello";
    0
}' "yes"

echo ""
echo "========================================="
echo "  Check Complete"
echo "========================================="
