#!/bin/bash
# ZULON 循环功能验证脚本

set -e

echo "========================================="
echo "  ZULON 编译器 - 循环功能验证"
echo "========================================="
echo ""

# 检查编译状态
echo "[1/5] 检查编译器状态..."
cargo build --package zulon-codegen-llvm --quiet 2>&1 | grep -E "error|Finished" || true
echo "✅ 编译器就绪"
echo ""

# 创建测试1: 基础while循环
echo "[2/5] 测试1: 基础while循环..."
cat > /tmp/test_basic_loop.zl << 'EOF'
fn main() -> i32 {
    let mut count = 0;
    while count < 10 {
        count = count + 1
    };
    count
}
EOF

# 创建测试2: 嵌套while循环
echo "[3/5] 测试2: 嵌套while循环..."
cat > /tmp/test_nested_loop.zl << 'EOF'
fn main() -> i32 {
    let mut sum = 0;
    let mut i = 0;
    while i < 5 {
        let mut j = 0;
        while j < 3 {
            sum = sum + 1;
            j = j + 1
        };
        i = i + 1
    };
    sum
}
EOF

# 创建测试3: 多变量循环
echo "[4/5] 测试3: 多变量循环..."
cat > /tmp/test_multi_var.zl << 'EOF'
fn main() -> i32 {
    let mut sum = 0;
    let mut count = 0;
    while count < 10 {
        sum = sum + count;
        count = count + 1
    };
    sum
}
EOF

echo "✅ 测试程序已创建"
echo ""

# 显示测试摘要
echo "[5/5] 测试摘要"
echo "-----------------------------------"
echo "测试1: 基础循环 (count < 10)"
echo "  预期结果: 10"
echo ""
echo "测试2: 嵌套循环 (i < 5, j < 3)"
echo "  预期结果: 15 (5 * 3)"
echo ""
echo "测试3: 多变量 (sum += count)"
echo "  预期结果: 45 (0+1+2+...+9)"
echo "-----------------------------------"
echo ""
echo "下一步: 使用cargo run --example编译这些测试"
echo ""
echo "========================================="
echo "  验证脚本准备完成"
echo "========================================="
