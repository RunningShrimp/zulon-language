#!/bin/bash
# ZULON 完整工程结构一键生成脚本
# 使用方法: ./setup_zulon_workspace.sh

set -e

echo "🚀 开始创建 ZULON 语言完整工程结构..."

# 1. 创建根目录
# mkdir -p zulon-lang && cd zulon-lang

# 2. 创建 Workspace Cargo.toml
cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = [
    # 编译器前端
    "crates/zulon-parser",
    "crates/zulon-resolver",
    "crates/zulon-typeck",
    "crates/zulon-mir",
    "crates/zulon-air",
    
    # 代码生成后端
    "crates/zulon-codegen-llvm",
    "crates/zulon-codegen-cranelift",
    "crates/zulon-codegen-wasm",
    "crates/zulon-codegen-jvm",
    "crates/zulon-codegen-js",
    "crates/zulon-codegen-rust",
    
    # 运行时
    "crates/zulon-runtime-core",
    "crates/zulon-runtime-memory",
    "crates/zulon-runtime-scheduler",
    "crates/zulon-runtime-effect",
    "crates/zulon-runtime-actor",
    "crates/zulon-runtime-io",
    "crates/zulon-runtime-net",
    
    # 标准库
    "crates/zulon-std-core",
    "crates/zulon-std-std",
    
    # 扩展库
    "crates/zulon-ext-serde",
    "crates/zulon-ext-test",
    "crates/zulon-ext-bench",
    
    # 工具链
    "crates/zulon-tools-yan",
    "crates/zulon-tools-lsp",
    "crates/zulon-tools-vet",
    "crates/zulon-tools-fmt",
    "crates/zulon-tools-repl",
    "crates/zulon-tools-profile",
    
    # 测试
    "crates/zulon-tests-unit",
    "crates/zulon-tests-integration",
    "crates/zulon-tests-benchmarks",
    "crates/zulon-tests-fuzz",
    
    # 其他
    "crates/zulon-build",
    "crates/zulon-ci",
]

# 默认成员（快速构建）
default-members = [
    "crates/zulon-tools-yan",
    "crates/zulon-std-core",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["ZULON Language Team"]
license = "Apache-2.0 OR MIT"
repository = "https://github.com/zulon-lang/zulon"
homepage = "https://zulon-lang.org"

[workspace.dependencies]
# 内部依赖
zulon-parser = { path = "crates/zulon-parser" }
zulon-resolver = { path = "crates/zulon-resolver" }
zulon-typeck = { path = "crates/zulon-typeck" }
zulon-mir = { path = "crates/zulon-mir" }
zulon-air = { path = "crates/zulon-air" }
zulon-codegen-llvm = { path = "crates/zulon-codegen-llvm" }

# 外部依赖（P0 最小集）
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
indexmap = "2.0"
smallvec = "1.11"

# P1+ 依赖
salsa = { version = "0.17", optional = true }
tower-lsp = { version = "0.20", optional = true }
inkwell = { version = "0.2", features = ["llvm16-0"], optional = true }

[profile.dev]
opt-level = 0
debug = true
debug-assertions = true

[profile.release]
opt-level = 3
debug = false
lto = true
codegen-units = 1

[profile.bench]
inherits = "release"
debug = true
EOF

# 3. 创建 rust-toolchain.toml
cat > rust-toolchain.toml << 'EOF'
[toolchain]
channel = "1.92.0"  # MSRV
components = ["rustfmt", "clippy", "rust-src", "llvm-tools"]
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "wasm32-wasi",
    "riscv32imc-unknown-none-elf"
]
EOF

# 4. 创建 yan.toml（ZULON 工具链配置）
cat > yan.toml << 'EOF'
[workspace]
root = "."

[profile.dev]
opt-level = 0
incremental = true

[profile.release]
opt-level = "speed"
lto = true
strip = true

[profile.test]
opt-level = 1

[registry]
default = "crates-io"

[build]
jobs = 0  # 使用所有 CPU
target-dir = "target"
EOF

# 5. 批量创建所有 Crate 目录
echo "📁 创建编译器前端 Crate..."
for crate in parser resolver typeck mir air; do
  mkdir -p "crates/zulon-$crate/src" "crates/zulon-$crate/tests"
  cat > "crates/zulon-$crate/Cargo.toml" << EOF
[package]
name = "zulon-$crate"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
zulon-std-core = { path = "../zulon-std-core" }
serde = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
zulon-ext-test = { path = "../zulon-ext-test" }
EOF
done

echo "📁 创建代码生成后端 Crate..."
for backend in llvm cranelift wasm jvm js rust; do
  mkdir -p "crates/zulon-codegen-$backend/src"
  cat > "crates/zulon-codegen-$backend/Cargo.toml" << EOF
[package]
name = "zulon-codegen-$backend"
version.workspace = true
edition.workspace = true

[dependencies]
zulon-air = { path = "../zulon-air" }
zulon-std-core = { path = "../zulon-std-core" }
EOF
done

echo "📁 创建运行时 Crate..."
for runtime in core memory scheduler effect actor io net; do
  mkdir -p "crates/zulon-runtime-$runtime/src" "crates/zulon-runtime-$runtime/tests"
  cat > "crates/zulon-runtime-$runtime/Cargo.toml" << EOF
[package]
name = "zulon-runtime-$runtime"
version.workspace = true
edition.workspace = true

[dependencies]
zulon-runtime-core = { path = "../zulon-runtime-core" }
smallvec = { workspace = true }
EOF
done

echo "📁 创建标准库 Crate..."
mkdir -p "crates/zulon-std-core/src"
cat > "crates/zulon-std-core/Cargo.toml" << 'EOF'
[package]
name = "zulon-std-core"
version.workspace = true
edition.workspace = true
description = "ZULON Core Library (no_std)"

[features]
default = []

[dependencies]
EOF

mkdir -p "crates/zulon-std-std/src"
cat > "crates/zulon-std-std/Cargo.toml" << 'EOF'
[package]
name = "zulon-std-std"
version.workspace = true
edition.workspace = true
description = "ZULON Standard Library"

[dependencies]
zulon-std-core = { path = "../zulon-std-core" }
zulon-runtime-effect = { path = "../zulon-runtime-effect" }
zulon-runtime-io = { path = "../zulon-runtime-io" }
EOF

echo "📁 创建扩展库 Crate..."
for ext in serde test bench; do
  mkdir -p "crates/zulon-ext-$ext/src"
  cat > "crates/zulon-ext-$ext/Cargo.toml" << EOF
[package]
name = "zulon-ext-$ext"
version.workspace = true
edition.workspace = true

[dependencies]
zulon-std-std = { path = "../zulon-std-std" }
EOF
done

echo "📁 创建工具链 Crate..."
mkdir -p "crates/zulon-tools-yan/src/commands"
cat > "crates/zulon-tools-yan/Cargo.toml" << 'EOF'
[package]
name = "yan"
version.workspace = true
edition.workspace = true
description = "ZULON Unified Toolchain"

[[bin]]
name = "yan"
path = "src/main.rs"

[dependencies]
zulon-parser = { path = "../zulon-parser" }
zulon-resolver = { path = "../zulon-resolver" }
zulon-codegen-llvm = { path = "../zulon-codegen-llvm" }
zulon-std-std = { path = "../zulon-std-std" }
clap = { version = "4.4", features = ["derive"] }
anyhow = { workspace = true }
EOF

for tool in lsp vet fmt repl profile; do
  mkdir -p "crates/zulon-tools-$tool/src"
  cat > "crates/zulon-tools-$tool/Cargo.toml" << EOF
[package]
name = "zulon-tools-$tool"
version.workspace = true
edition.workspace = true

[dependencies]
zulon-std-core = { path = "../zulon-std-core" }
EOF
done

echo "📁 创建测试 Crate..."
mkdir -p "crates/zulon-tests-unit/src"
cat > "crates/zulon-tests-unit/Cargo.toml" << 'EOF'
[package]
name = "zulon-tests-unit"
version.workspace = true
edition.workspace = true

[dependencies]
zulon-ext-test = { path = "../zulon-ext-test" }
EOF

mkdir -p "crates/zulon-tests-integration/tests"
cat > "crates/zulon-tests-integration/Cargo.toml" << 'EOF'
[package]
name = "zulon-tests-integration"
version.workspace = true
edition.workspace = true

[dependencies]
yan = { path = "../zulon-tools-yan" }
EOF

mkdir -p "crates/zulon-tests-benchmarks/benches"
cat > "crates/zulon-tests-benchmarks/Cargo.toml" << 'EOF'
[package]
name = "zulon-tests-benchmarks"
version.workspace = true
edition.workspace = true

[dependencies]
yan = { path = "../zulon-tools-yan" }
criterion = "0.5"

[[bench]]
name = "compiler_bench"
harness = false
EOF

mkdir -p "crates/zulon-tests-fuzz/fuzz_targets"
cat > "crates/zulon-tests-fuzz/Cargo.toml" << 'EOF'
[package]
name = "zulon-tests-fuzz"
version.workspace = true
edition.workspace = true

[dependencies]
libfuzzer-sys = "0.4"

[[bin]]
name = "fuzz_parser"
path = "fuzz_targets/parse.rs"
EOF

echo "📁 创建其他辅助目录..."
mkdir -p "crates/zulon-build/src"
mkdir -p "crates/zulon-ci/src"
mkdir -p "crates/zulon-docs/book/src"
mkdir -p "crates/zulon-examples"

# 6. 创建 .gitignore
cat > .gitignore << 'EOF'
# 构建产物
/target/
/target-*/
**/*.rs.bk
*.pdb

# IDE
.idea/
.vscode/settings.json.local
/*.code-workspace

# 操作系统
.DS_Store
Thumbs.db

# 测试输出
**/*.profraw
**/*.profdata
**/*.gcda
**/*.gcno
**/*.gcov

# 模糊测试
crates/zulon-tests-fuzz/artifacts/
crates/zulon-tests-fuzz/corpus/

# 发布包
*.tar.gz
*.zip
*.deb
*.rpm

# 临时文件
*.tmp
*.log
*~
EOF

# 7. 创建 GitHub Actions
mkdir -p .github/workflows
cat > .github/workflows/ci.yml << 'EOF'
name: ZULON CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo doc --no-deps

  test:
    name: Test
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo test --no-default-features

  coverage:
    name: Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo tarpaulin --all-features --workspace --timeout 120 --out xml
      - uses: codecov/codecov-action@v3
        with:
          file: cobertura.xml

  fuzz:
    name: Fuzz
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-fuzz
      - run: cargo fuzz run parse -- -max_total_time=3600
      - run: cargo fuzz run typeck -- -max_total_time=3600
EOF

# 8. 创建 README.md
cat > README.md << 'EOF'
# ZULON Programming Language

**The memory-safe, concurrent-safe systems language with zero-cost abstractions.**

[![CI](https://github.com/zulon-lang/zulon/workflows/ZULON%20CI/badge.svg)](https://github.com/zulon-lang/zulon/actions)
[![Coverage](https://codecov.io/gh/zulon-lang/zulon/branch/main/graph/badge.svg)](https://codecov.io/gh/zulon-lang/zulon)
[![License](https://img.shields.io/badge/license-Apache%202.0%20OR%20MIT-blue.svg)](LICENSE)

## Quick Start

### Installation
```bash
curl --proto '=https' --tlsv1.2 -sSf https://zulon-lang.org/install.sh | sh